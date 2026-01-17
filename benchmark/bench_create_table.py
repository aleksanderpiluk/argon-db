import argondb_pb2_grpc as rpc
import create_table_pb2 as create_pb
from bench_common import create_channel

def main():
    channel = create_channel()
    client = rpc.ArgonDbStub(channel)

    req = create_pb.CreateTableRequest(
        table_name="bench_table_0",
        columns=[
            create_pb.CreateTableRequestColumn(
                column_name="id",
                column_type=create_pb.Text,
            ),
            create_pb.CreateTableRequestColumn(
                column_name="value",
                column_type=create_pb.U16,
            ),
        ],
        primary_key=["id"],
    )

    client.CreateTable(req)
    print("Table created")

if __name__ == "__main__":
    main()
