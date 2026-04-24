import pandas as pd
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.lines import Line2D
from pathlib import Path

OUT_DIR = Path("results/plots")
OUT_DIR.mkdir(parents=True, exist_ok=True)

OPS = ["insert", "read", "scan"]

COLORS = {
    "insert": "tab:blue",
    "read": "tab:green",
    "scan": "tab:red",
}

OP_TITLES = {
    "insert": "Mutacje tabeli (Insert)",
    "read": "Odczyt pojedynczego wiersza (Read row)",
    "scan": "Odczyt zakresu wierszy (Scan)"
}

def plot_concurrency(df, concurrency):
    fig, axes = plt.subplots(3, 1, figsize=(8, 9), sharex=True)

    fig.suptitle(
        f"Obciążenie mieszane - {concurrency} równoległych klientów",
        fontsize=15
    )

    for ax, op in zip(axes, OPS):
        sub = df[df["op"] == op].sort_values("time_sec")

        if sub.empty:
            continue

        x = sub["time_sec"]

        # =========================
        # Throughput (RAW + TREND)
        # =========================

        thr = sub["throughput_ops_sec"]
        thr_smooth = thr.rolling(window=15, min_periods=1).mean()

        # RAW (noise)
        ax.plot(
            x,
            thr,
            color=COLORS[op],
            alpha=0.4,
            linewidth=1.2,
            label="przepustowość (wart. rzeczywiste)",
        )

        # TREND (main signal)
        ax.plot(
            x,
            thr_smooth,
            color=COLORS[op],
            alpha=1.0,
            linewidth=2.0,
            label="przepustowość (wygładzona)",
        )

        ax.set_ylabel("Przepustowość [operacje/s]")
        ax.set_ylim(0, thr.max() * 1.2)
        ax.grid(True, alpha=0.3)

        # =========================
        # Latency (right axis)
        # =========================
        ax2 = ax.twinx()

        ax2.plot(x, sub["mean_ms"], color="indigo", alpha=1, linewidth=1.5, label="opóźnienie (średnia arytmetyczna)")
        ax2.plot(x, sub["p95_ms"], color="mediumorchid", alpha=1, linewidth=1, linestyle="-", label="opóźnienie (95. centyl)")
        ax2.plot(x, sub["p99_ms"], color="deeppink", alpha=0.6, linewidth=0.75, linestyle="-", label="opóźnienie (99. centyl)")

        # shaded bands (stability visualization)
        ax2.fill_between(
            x,
            sub["mean_ms"],
            sub["p95_ms"],
            color="mediumorchid",
            alpha=0.1,
        )

        ax2.fill_between(
            x,
            sub["p95_ms"],
            sub["p99_ms"],
            color="deeppink",
            alpha=0.04,
        )

        ax2.set_ylabel("Opóźnienie [ms]")

        # tight latency scaling (prevents visual dominance)
        ax2.set_ylim(
            sub["mean_ms"].min() * 0.5,
            sub["p99_ms"].max() * 2.5
        )

        h1, l1 = ax.get_legend_handles_labels()
        h2, l2 = ax2.get_legend_handles_labels()

        ax.legend(
            h1 + h2,
            l1 + l2,
            loc="upper center",
            bbox_to_anchor=(0.5, -0.265),
            ncol=3,
            fontsize=8,
            frameon=True
        )

        ax.set_title(OP_TITLES[op])

        ax.set_xlabel("Czas [s]")
        ax.tick_params(labelbottom=True)

        ax.margins(x=0)
        ax2.margins(x=0)

    axes[-1].set_xlabel("Czas [s]")

    plt.tight_layout()

    plt.subplots_adjust(hspace=0.85, bottom=0.1075)

    out = OUT_DIR / f"mixed_{concurrency}_overview.png"
    plt.savefig(out, dpi=150)
    plt.close()

    print(f"Saved {out}")


def main():
    for c in [4, 8, 16, 32]:
        df = pd.read_csv(f"results/bench_mixed_{c}.csv")
        plot_concurrency(df, c)


if __name__ == "__main__":
    main()