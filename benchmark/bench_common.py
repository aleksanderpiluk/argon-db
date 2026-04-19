import time
import csv
import grpc
import os
from statistics import mean

def create_channel(target=None):
    target = target or os.getenv("BENCH_DB_HOST", "localhost:50051")

    return grpc.insecure_channel(
        target,
        options=[
            ("grpc.enable_http_proxy", 0),
            ("grpc.keepalive_time_ms", 10000),
        ],
    )

def write_csv(path, header, rows):
    with open(path, "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(header)
        writer.writerows(rows)


def percentile(sorted_data, p):
    n = len(sorted_data)
    if n == 0:
        return 0.0

    k = (n - 1) * p
    f = int(k)
    c = min(f + 1, n - 1)

    if f == c:
        return sorted_data[f]

    return sorted_data[f] + (sorted_data[c] - sorted_data[f]) * (k - f)


def latency_stats(latencies):
    if not latencies:
        return {
            "count": 0,
            "mean_ms": 0,
            "median_ms": 0,
            "p95_ms": 0,
            "p99_ms": 0,
        }

    latencies = sorted(latencies)

    return {
        "count": len(latencies),
        "mean_ms": mean(latencies) * 1000,
        "median_ms": percentile(latencies, 0.5) * 1000,  # p50
        "p95_ms": percentile(latencies, 0.95) * 1000,
        "p99_ms": percentile(latencies, 0.99) * 1000,
    }