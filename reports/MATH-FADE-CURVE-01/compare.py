"""Arithmetic illustrations for MATH-FADE-CURVE-01, not a production model.

All days, half-lives and shapes below are explicit comparison hypotheses.
Decimal approximates the real formulas; this does not certify Rust/libm or
the binary64 contract. No core, model, history store or network is accessed.
Default output is Markdown. Optional plotting uses an already installed
matplotlib; it is not a project dependency. Nothing is written by default.
"""

from __future__ import annotations

import argparse
from decimal import Decimal as D
from decimal import localcontext
from pathlib import Path


DAYS = (1, 30, 365, 1095)
FAMILIES = ("E", "P1", "S0.5")


def ratio(day: D, half: D, family: str) -> D:
    z = day / half
    if family == "E":
        return (-D(2).ln() * z).exp()
    if family == "P1":
        return D(1) / (D(1) + z)
    if family == "S0.5":
        return (-D(2).ln() * z.sqrt()).exp()
    raise ValueError(family)


def power(day: D, half: D, beta: D) -> D:
    c = (D(2).ln() / beta).exp() - 1
    return (-beta * (1 + c * day / half).ln()).exp()


def fmt(value: D) -> str:
    return f"{value:.9E}"


def table(head: list[str], rows: list[list[str]]) -> str:
    return "\n".join(
        ["| " + " | ".join(head) + " |",
         "| " + " | ".join("---" for _ in head) + " |"]
        + ["| " + " | ".join(row) + " |" for row in rows]
    )


def tables(precision: int) -> str:
    with localcontext() as ctx:
        ctx.prec = precision
        parts: list[str] = []
        half = D(30)
        rows = [[str(day)] + [fmt(ratio(D(day), half, f)) for f in FAMILIES]
                for day in DAYS]
        parts += ["## Ratios: hypothetical half-life 30 running days",
                  table(["day", *FAMILIES], rows)]
        rows = [[str(day)] + [fmt(-D("0.5") * ratio(D(day), half, f))
                              for f in FAMILIES] for day in DAYS]
        parts += ["## References w: hypothetical v = -0.5",
                  table(["day", *FAMILIES], rows)]
        rows = [[str(day)] + [fmt(power(D(day), half, D(beta)))
                              for beta in ("0.5", "1", "2")]
                for day in DAYS]
        parts += ["## Power shape sensitivity: same first half-life",
                  table(["day", "beta=0.5", "beta=1", "beta=2"], rows)]

        # One coordinate, x=h, b=0, |v|=1/2, no saturation/updates.
        # Natural recovery half-life = 0.25 day; daily factor is exactly 1/16.
        # m=|h|/|v|. These are real-model examples, not binary64 Phi outputs.
        post = {family: D(1) for family in FAMILIES}
        daily_rows: list[list[str]] = []
        for day in range(1, max(DAYS) + 1):
            row = [str(day), fmt(D(2) ** (-4 * day))]
            for family in FAMILIES:
                before = post[family] / 16
                post[family] = max(before, ratio(D(day), half, family))
                row += [fmt(before), fmt(post[family])]
            if day in DAYS:
                daily_rows.append(row)
        parts += ["## Never recall versus daily recall: normalized slot magnitude",
                  table(["day", "never", "E pre", "E post", "P1 pre",
                         "P1 post", "S pre", "S post"], daily_rows)]
        rows = [[str(h)] + [fmt(ratio(D(365), D(h), f)) for f in FAMILIES]
                for h in (7, 30, 365)]
        parts += ["## Scale sensitivity at 365 running days",
                  table(["half-life (days)", *FAMILIES], rows)]

        # Historical comparison routes, not selectable scales or learned defaults.
        source_half = D("0.25")
        rows = [[str(day)] + [fmt(ratio(D(day), source_half, f)) for f in FAMILIES]
                + [fmt(D(1) / (D(1) + D(2).ln() * D(day) / source_half))]
                for day in DAYS]
        parts += ["## Direct bindings: hypothetical source half-life 6 hours",
                  table(["day", "E K=1", "P1 K=1", "S0.5 K=1", "raw P1 alpha=1"], rows),
                  "Raw P1 alpha=1: K=" + fmt(D(1) / D(2).ln())
                  + "; first half-life (hours)=" + fmt(D(6) / D(2).ln())]
        rows = []
        for exponent in (1022, 1075):
            n = D(exponent)
            for label, z in (("E", n), ("P1", D(2) ** exponent - 1),
                             ("S0.5", n * n)):
                rows.append([str(exponent), label, fmt(z), fmt(z * half / 365)])
        parts += ["## Ideal binary64 scale landmarks: not a libm guarantee",
                  table(["N (r=2^-N)", "family", "z", "years at H=30 days"], rows)]
        return "\n\n".join(parts)


def plot(path: Path) -> None:
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    plt.rcParams.update({"font.family": "DejaVu Sans", "font.size": 10,
                         "svg.hashsalt": "MATH-FADE-CURVE-01"})
    labels = {"E": "E: exponential", "P1": "P: power, beta = 1",
              "S0.5": "S: stretched, nu = 0.5"}
    colors = {"E": "#2563eb", "P1": "#b45309", "S0.5": "#7c3aed"}
    fig, axes = plt.subplots(1, 2, figsize=(11.4, 4.1), layout="constrained")
    with localcontext() as ctx:
        ctx.prec = 60
        for ax, stop, start in ((axes[0], 60, 0), (axes[1], 1095, 30)):
            days = [D(start) + D(stop - start) * D(j) / 400 for j in range(401)]
            for family in FAMILIES:
                values = [float(ratio(day, D(30), family)) for day in days]
                ax.plot([float(d) for d in days], values, color=colors[family],
                        label=labels[family], linewidth=2.1)
                points = [d for d in DAYS if start <= d <= stop]
                ax.scatter(points, [float(ratio(D(d), D(30), family)) for d in points],
                           color=colors[family], s=23, zorder=3)
            ax.grid(True, alpha=0.2)
            ax.set_xlabel("Cumulative running days since target source")
            ax.set_ylabel("Recall reference ratio r")
            ax.spines[["top", "right"]].set_visible(False)
        axes[0].set(title="Early shape (linear scale)", xlim=(0, 60), ylim=(0, 1.02))
        axes[0].axvline(30, color="#64748b", linestyle=":", linewidth=1)
        axes[0].legend(frameon=False, loc="upper right")
        axes[1].set(title="Long tail (logarithmic vertical scale)", xlim=(30, 1095))
        axes[1].set_yscale("log")
        axes[1].set_ylim(1e-12, 1)
        axes[1].set_xticks([30, 365, 730, 1095])
        fig.suptitle("Illustration only: every curve first reaches 1/2 at 30 running days",
                     fontsize=12)
        fig.savefig(path, metadata={"Date": None} if path.suffix == ".svg" else {})
        plt.close(fig)
        if path.suffix == ".svg":
            # Matplotlib path data may end lines with spaces; keep the artifact clean.
            lines = path.read_text(encoding="utf-8").splitlines()
            path.write_text("\n".join(line.rstrip() for line in lines) + "\n",
                            encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--precision", type=int, default=80)
    parser.add_argument("--check-display", action="store_true")
    parser.add_argument("--plot", type=Path)
    args = parser.parse_args()
    if args.check_display:
        assert tables(80) == tables(120)
        print("80/120 decimal digits: all displayed 10-significant-digit values agree.")
    elif args.plot is not None:
        plot(args.plot)
        print(f"Wrote {args.plot}")
    else:
        print(tables(args.precision))


if __name__ == "__main__":
    main()
