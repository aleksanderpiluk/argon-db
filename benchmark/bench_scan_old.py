import time
from concurrent.futures import ThreadPoolExecutor, as_completed

import argondb_pb2_grpc as rpc
import scan_table_pb2 as scan_pb
from bench_common import create_channel, write_csv, latency_stats

TABLE = "bench_table_0"

CONCURRENCY = [1, 2, 4, 8, 16]
REQUESTS_PER_WORKER = 50

def worker(target):
    channel = create_channel(target)
    client = rpc.ArgonDbStub(channel)

    req = scan_pb.ScanTableRequest(table_name=TABLE)
    latencies = []

    for _ in range(REQUESTS_PER_WORKER):
        start = time.perf_counter()
        list(client.ScanTable(req).rows)
        latencies.append(time.perf_counter() - start)

    return latencies


def main(target="localhost:50051"):
    rows = []

    for workers in CONCURRENCY:
        start_wall = time.perf_counter()
        latencies = []

        with ThreadPoolExecutor(max_workers=workers) as pool:
            futures = [pool.submit(worker, target) for _ in range(workers)]
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
            f"[Scan] workers={workers} "
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
