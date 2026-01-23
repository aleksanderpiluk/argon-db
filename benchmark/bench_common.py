import time
import csv
import grpc
from statistics import mean, median

def create_channel(target="localhost:50051"):
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

def latency_stats(latencies):
    latencies = sorted(latencies)
    n = len(latencies)
    return {
        "count": n,
        "mean_ms": mean(latencies) * 1000,
        "median_ms": median(latencies) * 1000,
        "p95_ms": latencies[int(n * 0.95)] * 1000,
        "p99_ms": latencies[int(n * 0.99)] * 1000,
    }
