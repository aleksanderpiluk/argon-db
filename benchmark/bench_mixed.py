import os
import time
import uuid
import random
import threading
import csv
from concurrent.futures import ThreadPoolExecutor

from google.protobuf import struct_pb2

import argondb_pb2_grpc as rpc
import insert_mutations_pb2 as insert_pb
import read_row_pb2 as read_pb
import scan_table_pb2 as scan_pb
import create_table_pb2 as create_pb

from bench_common import create_channel, latency_stats

# =====================
# CONFIG
# =====================

CONCURRENCY = int(os.getenv("BENCH_CONCURRENCY", "8"))

DURATION_SEC = 300
STATS_INTERVAL = 1

WORKLOAD = {
    "insert": 0.2,
    "read":   0.6,
    "scan":   0.2,
}

RANGE_SIZE = 50
PK_COLUMN = "id"

ENABLE_SANITY = False

# =====================
# GLOBAL STATE
# =====================

inserted_counter = 0
insert_lock = threading.Lock()

stop_event = threading.Event()

# per-window stats
stats_lock = threading.Lock()
window_latencies = {
    "insert": [],
    "read": [],
    "scan": [],
}
window_counts = {
    "insert": 0,
    "read": 0,
    "scan": 0,
}


# =====================
# HELPERS
# =====================

def make_value(v: str):
    return struct_pb2.Value(string_value=v)


def make_number(v: int):
    return struct_pb2.Value(number_value=v)


def make_marker(value: str):
    return scan_pb.PrimaryKeyMarker(
        values={PK_COLUMN: make_value(value)}
    )


def safe_worker(table, worker_id):
    try:
        worker(table)
    except Exception as e:
        print(f"[WORKER {worker_id} CRASHED] {e}")
        raise

def create_table(table_name: str):
    channel = create_channel()
    client = rpc.ArgonDbStub(channel)

    req = create_pb.CreateTableRequest(
        table_name=table_name,
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


# =====================
# WORKER
# =====================

def worker(table: str):
    global inserted_counter

    channel = create_channel()
    client = rpc.ArgonDbStub(channel)

    ops = list(WORKLOAD.keys())
    weights = list(WORKLOAD.values())

    while not stop_event.is_set():
        op = random.choices(ops, weights)[0]

        # -------- INSERT --------
        if op == "insert":
            with insert_lock:
                i = inserted_counter
                inserted_counter += 1

            row_id = f"{i:012d}"

            req = insert_pb.InsertMutationsRequest(
                table_name=table,
                values={
                    "id": make_value(row_id),
                    "value": make_number(i % 65536),
                },
            )

            start = time.perf_counter()
            client.InsertMutations(req)
            dur = time.perf_counter() - start

            with stats_lock:
                window_latencies["insert"].append(dur)
                window_counts["insert"] += 1

        # -------- READ --------
        elif op == "read":
            current = inserted_counter
            if current == 0:
                continue

            i = random.randint(0, current - 1)
            row_id = f"{i:012d}"

            req = read_pb.ReadRowRequest(
                table_name=table,
                primary_key_values={PK_COLUMN: make_value(row_id)},
            )

            start = time.perf_counter()
            resp = client.ReadRow(req)
            dur = time.perf_counter() - start

            if ENABLE_SANITY:
                if not resp.values:
                    raise RuntimeError("empty read")
                if PK_COLUMN not in resp.values:
                    raise RuntimeError("missing PK")

            with stats_lock:
                window_latencies["read"].append(dur)
                window_counts["read"] += 1

        # -------- SCAN --------
        elif op == "scan":
            current = inserted_counter
            if current < RANGE_SIZE:
                continue

            start_i = random.randint(0, current - RANGE_SIZE)
            end_i = start_i + RANGE_SIZE

            start_id = f"{start_i:012d}"
            end_id = f"{end_i:012d}"

            req = scan_pb.ScanTableRequest(table_name=table)
            getattr(req, "from").CopyFrom(make_marker(start_id))
            getattr(req, "to").CopyFrom(make_marker(end_id))

            start = time.perf_counter()
            rows = list(client.ScanTable(req).rows)
            dur = time.perf_counter() - start

            if ENABLE_SANITY and not rows:
                raise RuntimeError("empty scan")

            with stats_lock:
                window_latencies["scan"].append(dur)
                window_counts["scan"] += 1


# =====================
# STATS LOOP
# =====================

def stats_loop(csv_writer):
    global window_latencies, window_counts

    start_time = time.time()
    last_time = start_time

    while not stop_event.is_set():
        time.sleep(STATS_INTERVAL)

        now = time.time()
        actual_interval = now - last_time
        last_time = now

        with stats_lock:
            lat_copy = {k: v[:] for k, v in window_latencies.items()}
            cnt_copy = window_counts.copy()

            # reset window
            window_latencies = {k: [] for k in window_latencies}
            window_counts = {k: 0 for k in window_counts}

        elapsed = time.time() - start_time
        total_rows = inserted_counter

        print(f"\n--- t={elapsed:.1f}s rows={total_rows} ---")

        for op in ["insert", "read", "scan"]:
            if cnt_copy[op] == 0:
                continue

            stats = latency_stats(lat_copy[op])
            throughput = cnt_copy[op] / actual_interval

            print(
                f"{op:6} "
                f"ops={cnt_copy[op]:6d} "
                f"thr={throughput:8.1f}/s "
                f"p50={stats['median_ms']:.2f}ms "
                f"p95={stats['p95_ms']:.2f}ms "
                f"p99={stats['p99_ms']:.2f}ms"
            )

            csv_writer.writerow([
                round(elapsed, 2),
                total_rows,
                op,
                cnt_copy[op],
                throughput,
                stats["mean_ms"],
                stats["median_ms"],
                stats["p95_ms"],
                stats["p99_ms"],
            ])


# =====================
# MAIN
# =====================

def main():
    table = f"bench_mix_{uuid.uuid4().hex[:8]}"

    print(f"Creating table: {table}")
    create_table(table)

    print(f"Running mixed workload with {CONCURRENCY} workers")

    csv_file = open(f"results/bench_mixed_{CONCURRENCY}.csv", "w", newline="")
    csv_writer = csv.writer(csv_file)

    csv_writer.writerow([
        "time_sec",
        "rows",
        "op",
        "ops",
        "throughput_ops_sec",
        "mean_ms",
        "median_ms",
        "p95_ms",
        "p99_ms",
    ])

    stats_thread = threading.Thread(
        target=stats_loop,
        args=(csv_writer,),
        daemon=True,
    )
    stats_thread.start()

    with ThreadPoolExecutor(max_workers=CONCURRENCY) as pool:
        for i in range(CONCURRENCY):
            pool.submit(safe_worker, table, i)

        time.sleep(DURATION_SEC)
        stop_event.set()

    csv_file.close()

    print("Benchmark finished")


if __name__ == "__main__":
    main()