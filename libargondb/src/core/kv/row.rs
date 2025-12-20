use std::{collections::BTreeMap, fmt::Debug};

use crate::kv::{
    KVRuntimeError, KVRuntimeErrorKind, KVScanIteratorItem, KVTableSchema,
    column_type::{
        ColumnType, ColumnTypeCode, ColumnTypeDeserialize, ColumnTypeText, ColumnTypeU16,
        ColumnTypeU16Array,
    },
    primary_key::{KVPrimaryKeyComparator, KVPrimaryKeySchema},
};

pub struct KVRow {
    table_schema: KVTableSchema,
    cells: BTreeMap<u16, Box<dyn KVScanIteratorItem + Send + Sync>>,
}

impl KVRow {
    pub fn column_deserialized<T>(
        &self,
        column_name: impl AsRef<str>,
    ) -> Result<T::Output, KVRuntimeError>
    where
        T: ColumnTypeDeserialize,
    {
        let column_schema = self
            .table_schema
            .lookup_by_name(column_name.as_ref())
            .ok_or(KVRuntimeError::with_msg(
                KVRuntimeErrorKind::OperationNotAllowed,
                format!("no column with name {}", column_name.as_ref()),
            ))?;

        let cell = self
            .cells
            .get(&column_schema.column_id)
            .ok_or(KVRuntimeError::with_msg(
                KVRuntimeErrorKind::OperationNotAllowed,
                format!("no cell for column {} in the row", column_name.as_ref()),
            ))?;

        T::deserialize(cell.mutation().value())
    }

    pub fn has_cell(&self, column_id: u16) -> bool {
        self.cells.contains_key(&column_id)
    }
}

impl Debug for KVRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("KVRow");
        let mut f_ref = &mut f;

        for column in &self.table_schema.columns {
            let name = &column.column_name;

            match self.cells.get(&column.column_id) {
                Some(ref item) => match column.column_type {
                    ColumnTypeCode::Bytes => {
                        f_ref = f_ref.field(name, &item.mutation().value());
                    }
                    ColumnTypeCode::Text => {
                        f_ref = f_ref.field(
                            name,
                            &ColumnTypeText::deserialize(item.mutation().value()).unwrap(),
                        );
                    }
                    ColumnTypeCode::U16 => {
                        f_ref = f_ref.field(
                            name,
                            &ColumnTypeU16::deserialize(item.mutation().value()).unwrap(),
                        );
                    }
                    ColumnTypeCode::U16Array => {
                        f_ref = f_ref.field(
                            name,
                            &ColumnTypeU16Array::deserialize(item.mutation().value()).unwrap(),
                        )
                    }
                },
                None => {
                    f_ref = f_ref.field(name, &"NULL");
                }
            }
        }

        f.finish()
    }
}

pub struct KVRowBuilder {
    table_schema: KVTableSchema,
    pk_schema: KVPrimaryKeySchema,
    primary_key: Box<[u8]>,
    cells: BTreeMap<u16, Box<dyn KVScanIteratorItem + Send + Sync>>,
}

impl KVRowBuilder {
    pub fn new(
        table_schema: KVTableSchema,
        item: Box<dyn KVScanIteratorItem + Send + Sync>,
    ) -> Self {
        let pk_schema = KVPrimaryKeySchema::from_columns_schema(&table_schema);

        let primary_key = item.primary_key().to_vec().into_boxed_slice();

        let mut cells = BTreeMap::new();
        cells.insert(item.mutation().column_id(), item);

        Self {
            table_schema,
            pk_schema,
            primary_key,
            cells,
        }
    }

    pub fn can_add(
        &self,
        item: &Box<dyn KVScanIteratorItem + Send + Sync>,
    ) -> Result<bool, KVRuntimeError> {
        KVPrimaryKeyComparator::eq(&self.pk_schema, &self.primary_key, item.primary_key())
    }

    pub fn add(
        &mut self,
        item: Box<dyn KVScanIteratorItem + Send + Sync>,
    ) -> Result<(), KVRuntimeError> {
        if !self.can_add(&item)? {
            return Err(KVRuntimeError::with_msg(
                KVRuntimeErrorKind::OperationNotAllowed,
                "cannot add mutation to this row",
            ));
        }

        self.cells.insert(item.mutation().column_id(), item);
        Ok(())
    }
}

impl Into<KVRow> for KVRowBuilder {
    fn into(self) -> KVRow {
        KVRow {
            table_schema: self.table_schema,
            cells: self.cells,
        }
    }
}
