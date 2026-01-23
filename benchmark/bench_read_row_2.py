import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from google.protobuf import struct_pb2

import argondb_pb2_grpc as rpc
import read_row_pb2 as read_pb
from bench_common import create_channel, write_csv, latency_stats

TABLE = "bench_table_0"

# Must match insert benchmark
CONCURRENCY = [1, 2, 4, 8, 16, 32]
REQUESTS_PER_WORKER = 20000

PK_COLUMN = "id"
ENABLE_SANITY_CHECKS = True


def make_pk(value: str):
    return {
        PK_COLUMN: struct_pb2.Value(string_value=value)
    }


def worker(worker_id: int, target: str):
    channel = create_channel(target)
    client = rpc.ArgonDbStub(channel)

    latencies = []

    for i in range(REQUESTS_PER_WORKER):
        row_id = f"{worker_id:04d}{i:08d}"

        req = read_pb.ReadRowRequest(
            table_name=TABLE,
            primary_key_values=make_pk(row_id),
        )

        start = time.perf_counter()
        resp = client.ReadRow(req)
        latencies.append(time.perf_counter() - start)

        if ENABLE_SANITY_CHECKS:
            # 1️⃣ must return something
            if not resp.values:
                raise RuntimeError(
                    f"ReadRow returned empty for PK={row_id}"
                )

            # 2️⃣ must contain PK
            if PK_COLUMN not in resp.values:
                raise RuntimeError(
                    f"ReadRow missing PK column for PK={row_id}"
                )

            # 3️⃣ PK must match
            pk_value = resp.values[PK_COLUMN].string_value
            if pk_value != row_id:
                raise RuntimeError(
                    f"ReadRow PK mismatch: expected={row_id}, got={pk_value}"
                )

    return latencies


def main(target="localhost:50051"):
    rows = []

    for workers in CONCURRENCY:
        start_wall = time.perf_counter()
        latencies = []

        with ThreadPoolExecutor(max_workers=workers) as pool:
            futures = [
                pool.submit(worker, w, target)
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
            f"[ReadRow] workers={workers} "
            f"throughput={throughput:.1f} ops/s "
            f"p95={stats['p95_ms']:.2f} ms"
        )

    write_csv(
        "results/read_row_2.csv",
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
