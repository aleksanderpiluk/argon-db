import pandas as pd
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from pathlib import Path

OUT_DIR = Path("results/plots")
OUT_DIR.mkdir(parents=True, exist_ok=True)

PHASES = ["insert", "read", "scan"]

METRICS = [
    ("throughput_ops_sec", "Throughput"),
    ("mean_latency_ms", "Mean"),
    ("p95_latency_ms", "P95"),
    ("p99_latency_ms", "P99"),
]

COLORS = ["tab:blue", "tab:orange", "tab:red", "tab:purple"]

OP_TITLES = {
    "insert": "Mutacje tabeli (100 000 operacji)",
    "scan": "Odczyt zakresu wierszy (50 000 operacji)",
    "read": "Odczyt pojedynczego wiersza (50 000 operacji)",
}


def plot_phase(df, phase):
    sub = df[df["phase"] == phase].sort_values("workers")

    workers = sub["workers"].values
    x = np.arange(len(workers))

    width = 0.2

    fig, ax1 = plt.subplots(figsize=(11, 5))

    # --------------------
    # Throughput (left axis)
    # --------------------
    ax1.bar(
        x - 1.5 * width,
        sub["throughput_ops_sec"],
        width=width,
        label="przepustowość (średnia arytmetyczna)",
        color=COLORS[0],
    )

    ax1.set_ylabel("Przepustowość [operacje/s]")
    ax1.set_xlabel("Liczba równoległych klientów")
    ax1.grid(True, axis="y", alpha=0.3)

    # --------------------
    # Latency (right axis)
    # --------------------
    ax2 = ax1.twinx()

    ax2.bar(
        x - 0.5 * width,
        sub["mean_latency_ms"],
        width=width,
        label="opóźnienie (średnia arytmetyczna)",
        color=COLORS[1],
    )

    ax2.bar(
        x + 0.5 * width,
        sub["p95_latency_ms"],
        width=width,
        label="opóźnienie (95. centyl)",
        color=COLORS[2],
    )

    ax2.bar(
        x + 1.5 * width,
        sub["p99_latency_ms"],
        width=width,
        label="opóźnienie (99. centyl)",
        color=COLORS[3],
    )

    ax2.set_ylabel("Opóźnienie [ms]")
    ax2.set_ylim(
        0,
        sub["p99_latency_ms"].max() * 2
    )

    # --------------------
    # X axis
    # --------------------
    ax1.set_xticks(x)
    ax1.set_xticklabels(workers)

    # --------------------
    # Legend merge
    # --------------------
    h1, l1 = ax1.get_legend_handles_labels()
    h2, l2 = ax2.get_legend_handles_labels()

    ax1.legend(h1 + h2, l1 + l2, loc="upper left")

    plt.title(OP_TITLES[phase])
    plt.tight_layout()

    out_path = OUT_DIR / f"{phase}_100k.png"
    plt.savefig(out_path, dpi=150)
    plt.close(fig)

    print(f"Saved {out_path}")


def main():
    df = pd.read_csv("results/bench_100k.csv")

    for phase in PHASES:
        plot_phase(df, phase)


if __name__ == "__main__":
    main()