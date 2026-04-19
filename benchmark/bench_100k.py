import time
import threading
from concurrent.futures import ThreadPoolExecutor, as_completed
from google.protobuf import struct_pb2

import argondb_pb2_grpc as rpc
import create_table_pb2 as create_pb
import insert_mutations_pb2 as insert_pb
import read_row_pb2 as read_pb
import scan_table_pb2 as scan_pb

from bench_common import create_channel, write_csv, latency_stats

# =====================
# CONFIG
# =====================

CONCURRENCY = [1, 2, 4, 8, 16, 32]

TOTAL_INSERTS = 100_000
TOTAL_READS = 50_000
TOTAL_SCANS = 50_000

RANGE_SIZE = 50
PK_COLUMN = "id"

ENABLE_SANITY = True
WARMUP_OPS = 0

# =====================
# SHARED COUNTER
# =====================

class Counter:
    def __init__(self, limit):
        self.value = 0
        self.limit = limit
        self.lock = threading.Lock()

    def next(self):
        with self.lock:
            if self.value >= self.limit:
                return None
            v = self.value
            self.value += 1
            return v


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

# =====================
# CREATE TABLE UTIL FUNC
# =====================

def util_create_table(table_name: str):
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
# INSERT PHASE
# =====================

def insert_worker(worker_id, table, counter: Counter):
    channel = create_channel()
    client = rpc.ArgonDbStub(channel)

    latencies = []
    done = 0

    while True:
        i = counter.next()
        if i is None:
            break

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

        if done >= WARMUP_OPS:
            latencies.append(dur)

        done += 1

    return latencies, done


# =====================
# READ ROW PHASE
# =====================

def read_worker(worker_id, table, counter: Counter):
    channel = create_channel()
    client = rpc.ArgonDbStub(channel)

    latencies = []
    done = 0

    while True:
        i = counter.next()
        if i is None:
            break

        row_id = f"{i:012d}"

        req = read_pb.ReadRowRequest(
            table_name=table,
            primary_key_values={PK_COLUMN: make_value(row_id)},
        )

        start = time.perf_counter()
        resp = client.ReadRow(req)
        dur = time.perf_counter() - start
        latencies.append(dur)

        if ENABLE_SANITY:
            if not resp.values:
                raise RuntimeError(f"empty read for {row_id}")
            if PK_COLUMN not in resp.values:
                raise RuntimeError("missing PK")
            if resp.values[PK_COLUMN].string_value != row_id:
                raise RuntimeError("PK mismatch")

        done += 1

    return latencies, done


# =====================
# SCAN PHASE
# =====================

def scan_worker(worker_id, table, counter: Counter):
    channel = create_channel()
    client = rpc.ArgonDbStub(channel)

    latencies = []
    done = 0

    while True:
        i = counter.next()
        if i is None:
            break

        start_i = (i * RANGE_SIZE) % TOTAL_INSERTS
        end_i = start_i + RANGE_SIZE

        start_id = f"{start_i:012d}"
        end_id = f"{end_i:012d}"

        req = scan_pb.ScanTableRequest(table_name=table)
        getattr(req, "from").CopyFrom(make_marker(start_id))
        getattr(req, "to").CopyFrom(make_marker(end_id))

        start = time.perf_counter()
        rows = list(client.ScanTable(req).rows)
        dur = time.perf_counter() - start
        latencies.append(dur)

        if ENABLE_SANITY:
            if not rows:
                raise RuntimeError("empty scan")
            pk = rows[0].values[PK_COLUMN].string_value
            if not (start_id <= pk < end_id):
                raise RuntimeError("range violation")

        done += 1

    return latencies, done

# =====================
# RUNNER
# =====================

def run_phase(name, workers, fn):
    latencies = []
    total_ops = 0

    start_wall = time.perf_counter()

    with ThreadPoolExecutor(max_workers=workers) as pool:
        futures = [pool.submit(fn, w) for w in range(workers)]
        for f in as_completed(futures):
            lats, ops = f.result()
            latencies.extend(lats)
            total_ops += ops

    duration = time.perf_counter() - start_wall
    throughput = total_ops / duration
    stats = latency_stats(latencies)

    print(
        f"[{name}] workers={workers} "
        f"ops={total_ops} "
        f"thr={throughput:.1f}/s "
        f"p95={stats['p95_ms']:.2f}ms"
    )

    return [
        workers,
        name,
        total_ops,
        duration,
        throughput,
        stats["mean_ms"],
        stats["p95_ms"],
        stats["p99_ms"],
    ]


# =====================
# MAIN PIPELINE
# =====================

def main():
    rows = []

    for workers in CONCURRENCY:
        table = f"bench_{workers}_{int(time.time())}"

        print(f"\n=== PIPELINE workers={workers} table={table} ===")

        util_create_table(table)

        # -------- INSERT --------
        counter = Counter(TOTAL_INSERTS)

        def insert_fn(w):
            return insert_worker(w, table, counter)

        rows.append(run_phase("insert", workers, insert_fn))

        # -------- READ --------
        read_counter = Counter(TOTAL_READS)

        def read_fn(w):
            return read_worker(w, table, read_counter)

        rows.append(run_phase("read", workers, read_fn))


        # -------- SCAN --------
        scan_counter = Counter(TOTAL_SCANS)

        def scan_fn(w):
            return scan_worker(w, table, scan_counter)

        rows.append(run_phase("scan", workers, scan_fn))

    write_csv(
        "results/bench_100k.csv",
        [
            "workers",
            "phase",
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