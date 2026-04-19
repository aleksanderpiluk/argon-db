import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from google.protobuf import struct_pb2

import argondb_pb2_grpc as rpc
import scan_table_pb2 as scan_pb
from bench_common import create_channel, write_csv, latency_stats

TABLE = "bench_table_0"

# Tune these
CONCURRENCY = [1, 2, 4, 8, 16, 32]
# CONCURRENCY = [1]
REQUESTS_PER_WORKER = 100
RANGE_SIZE = 50        # rows per scan
KEY_SPACE_STRIDE = 1_000_000  # avoid overlapping ranges

PK_COLUMN = "id"
ENABLE_SANITY_CHECKS = True


def make_marker(value: str) -> scan_pb.PrimaryKeyMarker:
    return scan_pb.PrimaryKeyMarker(
        values={
            PK_COLUMN: struct_pb2.Value(string_value=value)
        }
    )


def make_range(worker_id: int, iteration: int):
    start_i = iteration * RANGE_SIZE
    end_i = start_i + RANGE_SIZE

    start_id = f"{worker_id:04d}{start_i:08d}"
    end_id = f"{worker_id:04d}{end_i:08d}"

    return start_id, end_id


def worker(worker_id: int):
    channel = create_channel()
    client = rpc.ArgonDbStub(channel)

    latencies = []

    for i in range(REQUESTS_PER_WORKER):
        start_id, end_id = make_range(worker_id, i)

        req = scan_pb.ScanTableRequest(
            table_name=TABLE,
        )

        getattr(req, "from").CopyFrom(make_marker(start_id))
        getattr(req, "to").CopyFrom(make_marker(end_id))

        start = time.perf_counter()
        rows = list(client.ScanTable(req).rows)
        latencies.append(time.perf_counter() - start)

        if ENABLE_SANITY_CHECKS:
            # 1️⃣ must return something
            if not rows:
                raise RuntimeError(
                    f"Scan returned no rows for range "
                    f"[{start_id}, {end_id})"
                )

            # 2️⃣ first row must contain PK
            row = rows[0]
            if PK_COLUMN not in row.values:
                print(row)
                raise RuntimeError(
                    f"Scan row missing PK column '{PK_COLUMN}'"
                )

            # 3️⃣ PK must be in range
            pk_value = row.values[PK_COLUMN].string_value
            if not (start_id <= pk_value < end_id):
                raise RuntimeError(
                    f"PK value {pk_value} outside range "
                    f"[{start_id}, {end_id})"
                )

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
            f"[Scan(range)] workers={workers} "
            f"throughput={throughput:.1f} ops/s "
            f"p95={stats['p95_ms']:.2f} ms"
        )

    write_csv(
        "results/scan.csv",
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
