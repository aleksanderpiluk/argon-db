import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from google.protobuf import struct_pb2

import argondb_pb2_grpc as rpc
import insert_mutations_pb2 as insert_pb
from bench_common import create_channel, write_csv, latency_stats

TABLE = "bench_table_0"

CONCURRENCY = [1, 2, 4, 8, 16, 32]
REQUESTS_PER_WORKER = 5000

def worker(worker_id):
    channel = create_channel()
    client = rpc.ArgonDbStub(channel)

    latencies = []

    for i in range(REQUESTS_PER_WORKER):
        row_id = f"{worker_id:04d}{i:08d}"

        req = insert_pb.InsertMutationsRequest(
            table_name=TABLE,
            values={
                "id": struct_pb2.Value(
                    string_value=row_id
                ),
                "value": struct_pb2.Value(
                    number_value=i % 65536
                ),
            },
        )

        start = time.perf_counter()
        client.InsertMutations(req)
        latencies.append(time.perf_counter() - start)

    return latencies


def main():
    rows = []

    for workers in CONCURRENCY:
        start_wall = time.perf_counter()
        latencies = []

        with ThreadPoolExecutor(max_workers=workers) as pool:
            futures = [
                pool.submit(worker, w)
                for w in range(workers)
            ]
            for f in as_completed(futures):
                latencies.extend(f.result())

        duration = time.perf_counter() - start_wall
        total_ops = workers * REQUESTS_PER_WORKER
        throughput = total_ops / duration
        stats = latency_stats(latencies)

        rows.append([
            workers,
            total_ops,
            duration,
            throughput,
            stats["mean_ms"],
            stats["p95_ms"],
            stats["p99_ms"],
        ])

        print(
            f"[Insert] workers={workers} "
            f"throughput={throughput:.1f} ops/s "
            f"p95={stats['p95_ms']:.2f} ms"
        )

    write_csv(
        "results/insert.csv",
        [
            "workers",
            "total_ops",
            "duration_sec",
            "throughput_ops_sec",
            "mean_latency_ms",
            "p95_latency_ms",
            "p99_latency_ms",
        ],
        rows,
    )

if __name__ == "__main__":
    main()
