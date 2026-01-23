import pandas as pd
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from pathlib import Path

OUT_DIR = Path("results/plots")
OUT_DIR.mkdir(parents=True, exist_ok=True)

def plot(csv_path, title, out_name):
    df = pd.read_csv(csv_path)

    fig, ax1 = plt.subplots(figsize=(10, 5))

    # Throughput (left axis)
    line1, = ax1.plot(
        df["workers"],
        df["throughput_ops_sec"],
        marker="o",
        label="Throughput (ops/sec)",
    )
    ax1.set_xlabel("Concurrent Clients")
    ax1.set_ylabel("Throughput (ops/sec)")
    ax1.grid(True)

    # Latency (right axis)
    ax2 = ax1.twinx()
    line2, = ax2.plot(
        df["workers"],
        df["p95_latency_ms"],
        marker="x",
        color="red",
        label="P95 latency (ms)",
    )
    ax2.set_ylabel("P95 Latency (ms)")

    # Combined legend
    lines = [line1, line2]
    labels = [l.get_label() for l in lines]
    ax1.legend(lines, labels, loc="lower right",)

    plt.title(title)
    plt.tight_layout()

    out_path = OUT_DIR / out_name
    plt.savefig(out_path, dpi=150)
    plt.close(fig)

    print(f"Saved {out_path}")

if __name__ == "__main__":
    plot("results/insert.csv", "InsertMutations - 5k rows per worker", "insert.png")
    plot("results/insert_2.csv", "InsertMutations 29k rows per worker", "insert_2.png")
    plot("results/scan.csv", "ScanTable - 5k rows per worker", "scan.png")
    plot("results/scan_2.csv", "ScanTable - 20k rows per worker", "scan_2.png")
    plot("results/read_row.csv", "ReadRow - 1k rows per worker", "read_row.png")
    plot("results/read_row_2.csv", "ReadRow - 20k rows per worker", "read_row_2.png")
