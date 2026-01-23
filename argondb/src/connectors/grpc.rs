use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use libargonconnector_grpc::argondb_service_definition::{
    self, ColumnDefinition, CreateTableRequest, InsertMutationsRequest, InsertMutationsResponse,
    ListTablesResponse, MutateRowRequest, PrimaryKeyMarker, ReadRowRequest, ReadRowResponse,
    ScanTableRequest, ScanTableResponse, ScanTableResponseRow, Table,
    argon_db_server::ArgonDbServer,
};
use libargondb::{
    ConnectorError, ConnectorHandle, DbCtx,
    kv::{
        KVColumnFilter, KVColumnValue, KVColumnValueBuilder, KVPrimaryKeyMarker, KVRangeScan,
        KVRow, KVRowScan, KVTable, KVTableName, KVTableSchema,
        column_type::{
            ColumnTypeBytes, ColumnTypeCode, ColumnTypeText, ColumnTypeU16, ColumnTypeU16Array,
        },
        primary_key::{KVPrimaryKeySchema, PrimaryKeyBuilder},
        schema::KVColumnSchema,
    },
};
use prost_types::{ListValue, Value, value::Kind};
use tokio::{runtime::Runtime, sync::oneshot, task::JoinHandle};
use tonic::{Request, Response, Status, transport::Server};

use crate::ops::{CreateTableOp, CreateTableOpColumn, CreateTableOpError, InsertIntoOp};

pub fn init_connector_grpc(db_ctx: Arc<DbCtx>) -> Result<Box<dyn ConnectorHandle>, ConnectorError> {
    const DEFAULT_PORT: u16 = 50051;
    const IP_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    const SOCKET_ADDR: SocketAddr = SocketAddr::new(IP_ADDR, DEFAULT_PORT);

    let (tx, rx) = oneshot::channel::<()>();

    let shutdown_fn = async move || {
        rx.await.unwrap();
        println!("grpc connector - shutdown signal received");
    };

    let runtime =
        Runtime::new().map_err(|e| ConnectorError::UnexpectedError(Arc::new(Box::from(e))))?;

    // tracing_subscriber::fmt()
    //     .with_env_filter(
    //         EnvFilter::from_default_env()
    //             .add_directive("tonic=debug".parse().unwrap())
    //             .add_directive("tower=debug".parse().unwrap())
    //             .add_directive("hyper=debug".parse().unwrap())
    //             .add_directive("h2=debug".parse().unwrap()),
    //     )
    //     .init();

    // let layer = TraceLayer::new_for_grpc().make_span_with(|request: &http::Request<_>| {
    //     tracing::span!(
    //         Level::DEBUG,
    //         "grpc_request",
    //         method = %request.method(),
    //         uri = %request.uri(),
    //         headers = ?request.headers()
    //     )
    // });

    println!(
        "gprc connector - starting server on address {}",
        SOCKET_ADDR
    );

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(argondb_service_definition::FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    let join_handle = runtime.spawn(async move {
        println!("grpc server task spawned");
        Server::builder()
            // .layer(layer)
            .add_service(reflection_service)
            .add_service(ArgonDbServer::new(ArgonDbHandlers::new(db_ctx)))
            .serve_with_shutdown(SOCKET_ADDR, shutdown_fn())
            .await
            .map_err(|err| ConnectorError::UnexpectedError(Arc::new(Box::from(err))))
    });

    Ok(Box::from(GrpcConnectorHandle {
        runtime,
        join_handle,
        tx,
    }))
}

struct GrpcConnectorHandle {
    runtime: Runtime,
    join_handle: JoinHandle<Result<(), ConnectorError>>,
    tx: tokio::sync::oneshot::Sender<()>,
}

#[async_trait]
impl ConnectorHandle for GrpcConnectorHandle {
    async fn close(self: Box<Self>) {
        println!("closing grpc connector");
        self.tx.send(()).unwrap();

        self.runtime.shutdown_timeout(Duration::from_secs(15));
        println!("grpc connector - tokio runtime closed");
    }
}

pub struct ArgonDbHandlers {
    db_ctx: Arc<DbCtx>,
}

impl ArgonDbHandlers {
    pub fn new(db_ctx: Arc<DbCtx>) -> Self {
        Self { db_ctx }
    }
}

#[async_trait]
impl argondb_service_definition::argon_db_server::ArgonDb for ArgonDbHandlers {
    async fn create_table(
        &self,
        request: Request<CreateTableRequest>,
    ) -> Result<Response<Table>, Status> {
        let req = request.get_ref();

        let op = CreateTableOp {
            table_name: req.table_name.clone(),
            columns: req
                .columns
                .iter()
                .map(|col| CreateTableOpColumn {
                    column_name: col.column_name.clone(),
                    column_type: match col.column_type {
                        0 => ColumnTypeCode::Bytes,
                        1 => ColumnTypeCode::Text,
                        2 => ColumnTypeCode::U16,
                        3 => ColumnTypeCode::U16Array,
                        _ => unreachable!(),
                    },
                })
                .collect(),
            primary_key: req.primary_key.clone(),
        };

        match op.execute(&self.db_ctx).await {
            Ok(table) => Ok(tonic::Response::new(Table {
                table_name: table.table_name.to_string(),
                columns: table
                    .table_schema
                    .columns
                    .iter()
                    .map(|col| ColumnDefinition {
                        column_name: col.column_name.clone(),
                    })
                    .collect(),
            })),
            Err(e) => match e {
                CreateTableOpError::InvalidTableName => {
                    Err(Status::invalid_argument("invalid table name"))
                }
                CreateTableOpError::PrimaryKeyColumnsCountExceeded => Err(
                    Status::invalid_argument("primary key max column count exceeded"),
                ),
                CreateTableOpError::PrimaryKeyInvalidColumn => {
                    Err(Status::invalid_argument("primary key invalid column"))
                }
                CreateTableOpError::PrimaryKeyMissing => {
                    Err(Status::invalid_argument("primary key missing"))
                }
                CreateTableOpError::TooManyColumns => {
                    Err(Status::invalid_argument("table max column count exceeded"))
                }
                CreateTableOpError::SchemaError => Err(Status::invalid_argument("schema error")),
            },
        }
    }

    async fn list_tables(&self, _: Request<()>) -> Result<Response<ListTablesResponse>, Status> {
        let tables = self.db_ctx.catalog.list_tables();

        Ok(tonic::Response::new(ListTablesResponse {
            tables: tables
                .into_iter()
                .map(|table| Table {
                    table_name: table.table_name.to_string(),
                    columns: table
                        .table_schema
                        .columns
                        .iter()
                        .map(|col| ColumnDefinition {
                            column_name: col.column_name.clone(),
                        })
                        .collect(),
                })
                .collect(),
        }))
    }

    async fn scan_table(
        &self,
        request: Request<ScanTableRequest>,
    ) -> Result<Response<ScanTableResponse>, Status> {
        let req = request.get_ref();

        let table_name = KVTableName::from_str(&req.table_name)
            .map_err(|_| Status::invalid_argument("invalid table name"))?;

        let table = self
            .db_ctx
            .catalog
            .lookup_table_by_name(&table_name)
            .ok_or(Status::not_found(format!(
                "table {} does not exist",
                table_name
            )))?;

        let pk_schema = KVPrimaryKeySchema::from_table_schema(&table.table_schema);

        let from: KVPrimaryKeyMarker = req
            .from
            .clone()
            .map(|PrimaryKeyMarker { values }| {
                let mut pk_builder = PrimaryKeyBuilder::new(&pk_schema);

                for column_id in &table.table_schema.primary_key {
                    let column_schema = table.table_schema.lookup_by_column_id(*column_id).unwrap();

                    let value = values.get(&column_schema.column_name).unwrap();

                    let pk_col_val = GrpcHandlerUtils::value_to_column_value(value).unwrap();
                    pk_builder.add_value(&pk_col_val.serialize().unwrap());
                }
                let primary_key = pk_builder.build();

                KVPrimaryKeyMarker::Key(primary_key)
            })
            .unwrap_or(KVPrimaryKeyMarker::Start);

        let to: KVPrimaryKeyMarker = req
            .to
            .clone()
            .map(|PrimaryKeyMarker { values }| {
                let mut pk_builder = PrimaryKeyBuilder::new(&pk_schema);

                for column_id in &table.table_schema.primary_key {
                    let column_schema = table.table_schema.lookup_by_column_id(*column_id).unwrap();

                    let value = values.get(&column_schema.column_name).unwrap();

                    let pk_col_val = GrpcHandlerUtils::value_to_column_value(value).unwrap();
                    pk_builder.add_value(&pk_col_val.serialize().unwrap());
                }
                let primary_key = pk_builder.build();

                KVPrimaryKeyMarker::Key(primary_key)
            })
            .unwrap_or(KVPrimaryKeyMarker::End);

        let map_scan_err = |e: libargondb::kv::KVRuntimeError| {
            println!("scan failed - {}", e);

            Status::internal("scan failed")
        };

        let mut scan = table
            .scan(KVRangeScan::new(
                table.table_schema.clone(),
                from,
                to,
                KVColumnFilter::All,
            ))
            .await
            .map_err(map_scan_err)?;

        let mut rows = Vec::<ScanTableResponseRow>::new();

        while let Some(row) = scan.next_row().await.map_err(map_scan_err)? {
            rows.push(ScanTableResponseRow {
                values: GrpcHandlerUtils::row_to_values_map(&table.table_schema, row),
            });
        }

        Ok(tonic::Response::new(ScanTableResponse { rows }))
    }

    async fn insert_mutations(
        &self,
        request: Request<InsertMutationsRequest>,
    ) -> Result<Response<InsertMutationsResponse>, Status> {
        let req = request.get_ref();

        let table_name = KVTableName::from_str(&req.table_name)
            .map_err(|_| Status::invalid_argument("invalid table name"))?;

        let table = self
            .db_ctx
            .catalog
            .lookup_table_by_name(&table_name)
            .ok_or(Status::not_found(format!(
                "table {} does not exist",
                table_name
            )))?;

        let mut values = vec![];

        for (key, val) in req.values.iter() {
            values.push((
                key.to_string(),
                GrpcHandlerUtils::value_to_column_value(val)
                    .map_err(|_| Status::invalid_argument("cannot map value to column"))?,
            ))
        }

        InsertIntoOp {
            table_name: table_name.to_string(),
            values,
        }
        .execute(&self.db_ctx)
        .await
        .map_err(|e| {
            println!("insert failed - {:?}", e);

            Status::internal("insert failed")
        })?;

        Ok(tonic::Response::new(InsertMutationsResponse {}))
    }

    async fn read_row(
        &self,
        request: Request<ReadRowRequest>,
    ) -> Result<Response<ReadRowResponse>, Status> {
        let req = request.get_ref();

        let table_name = KVTableName::from_str(&req.table_name)
            .map_err(|_| Status::invalid_argument("invalid table name"))?;

        let table = self
            .db_ctx
            .catalog
            .lookup_table_by_name(&table_name)
            .ok_or(Status::not_found(format!(
                "table {} does not exist",
                table_name
            )))?;

        let pk_schema = KVPrimaryKeySchema::from_table_schema(&table.table_schema);

        let values = req.primary_key_values.clone();
        let mut pk_builder = PrimaryKeyBuilder::new(&pk_schema);

        for column_id in &table.table_schema.primary_key {
            let column_schema = table.table_schema.lookup_by_column_id(*column_id).unwrap();

            let value = values.get(&column_schema.column_name).unwrap();

            let pk_col_val = GrpcHandlerUtils::value_to_column_value(value).unwrap();
            pk_builder.add_value(&pk_col_val.serialize().unwrap());
        }
        let primary_key = pk_builder.build();

        let map_scan_err = |e: libargondb::kv::KVRuntimeError| {
            println!("scan failed - {}", e);

            Status::internal("scan failed")
        };

        let mut scan = table
            .scan(KVRowScan::new(
                table.table_schema.clone(),
                primary_key,
                KVColumnFilter::All,
            ))
            .await
            .map_err(map_scan_err)?;

        let maybe_row = scan.next_row().await.map_err(map_scan_err)?;

        Ok(tonic::Response::new(ReadRowResponse {
            values: if let Some(row) = maybe_row {
                GrpcHandlerUtils::row_to_values_map(&table.table_schema, row)
            } else {
                HashMap::new()
            },
        }))
    }

    async fn mutate_row(&self, request: Request<MutateRowRequest>) -> Result<Response<()>, Status> {
        todo!()
    }
}

struct GrpcHandlerUtils;

impl GrpcHandlerUtils {
    fn row_to_values_map(schema: &KVTableSchema, row: KVRow) -> HashMap<String, Value> {
        let mut values = HashMap::new();

        for column in &schema.columns {
            if row.has_cell(column.column_id) {
                let value = GrpcHandlerUtils::cell_to_value(column, &row);

                values.insert(column.column_name.clone(), value);
            }
        }

        values
    }

    fn cell_to_value(schema: &KVColumnSchema, row: &KVRow) -> Value {
        match schema.column_type {
            ColumnTypeCode::Bytes => {
                let val = row
                    .column_deserialized::<ColumnTypeBytes>(&schema.column_name)
                    .unwrap();
                todo!()
            }
            ColumnTypeCode::Text => {
                let val = row
                    .column_deserialized::<ColumnTypeText>(&schema.column_name)
                    .unwrap();
                Value {
                    kind: Some(Kind::StringValue(val)),
                }
            }
            ColumnTypeCode::U16 => {
                let val = row
                    .column_deserialized::<ColumnTypeU16>(&schema.column_name)
                    .unwrap();
                Value {
                    kind: Some(Kind::NumberValue(val.into())),
                }
            }
            ColumnTypeCode::U16Array => {
                let val = row
                    .column_deserialized::<ColumnTypeU16Array>(&schema.column_name)
                    .unwrap();
                Value {
                    kind: Some(Kind::ListValue(ListValue {
                        values: val
                            .into_iter()
                            .map(|val| Value {
                                kind: Some(Kind::NumberValue(val.into())),
                            })
                            .collect(),
                    })),
                }
            }
        }
    }

    fn value_to_column_value(
        value: &Value,
    ) -> Result<Box<(dyn KVColumnValue + Send + Sync + 'static)>, ()> {
        let Some(kind) = &value.kind else {
            return Err(());
        };

        match kind {
            Kind::StringValue(value) => Ok(KVColumnValueBuilder::text(value.clone())),
            Kind::NumberValue(value) => Ok(KVColumnValueBuilder::u16(*value as u16)),
            _ => Err(()),
        }
    }
}
