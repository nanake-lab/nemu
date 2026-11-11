"""Materialize frozen resources, six probe histories and full cost ledger.

Preparation only: no DUT/model execution. Full histories beyond the six §8
histories are recipes tied to the immutable prereg; expansion/execution must
wait for the resource gate. All output goes to the private evidence directory.
"""

import argparse
import hashlib
import itertools
import json
from pathlib import Path
from typing import Any

BASE = "8839dcae73c98437f61222f49d08cafd9ba8f4e6"
PREREG = "e58542622da8a33a17d12f26dd42a5b4e699a5cc"


def write(path: Path, data: Any) -> None:
    path.write_text(
        json.dumps(data, ensure_ascii=False, sort_keys=True, indent=2) + "\n",
        encoding="utf-8",
    )


def binding(path: Path) -> dict[str, Any]:
    b = path.read_bytes()
    return dict(sha256=hashlib.sha256(b).hexdigest(), length=len(b))


def resources() -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
    c = dict(
        version="P5-71-v2-C",
        gain_prior=1,
        sigma_prior=0.3,
        tau_prior_s=10,
        baseline_weight=2,
        baseline_prior=0,
        variance_weight=10,
        duration_weight=10,
        change_prior=0.05,
        change_prior_weight=10,
        change_min=0.01,
        gain_min=0.2,
        gain_max=3,
        recovery_min=0.001,
        recovery_max=2,
        detector_shift_sigma=1,
        detector_threshold=8,
        jump_notice=100,
        drift_run=1000000,
        drift_margin=0.01,
    )
    f = dict(
        version="P5-71-v2-F",
        g_max=1,
        r_max=1,
        w_min=0.1,
        b=0,
        g=1,
        lambda_bits="3fe62e42fefa39ef",
        kappa=0.5,
        max_bytes=1048576,
        max_items=4096,
        max_depth=32,
    )
    g = dict(
        version="P5-71-v2-G",
        schema_version=1,
        n0=3,
        kappa1=1,
        w_max=8,
        w_teach=1,
        v_u=0.5,
        r_min=0.05,
        r_max=1,
        h0=0.1,
        n_h=10,
        h_hat_min=None,
        h0_theta=0.1,
        h_hat_theta_min=0.01,
        n_b=10,
        n_sigma=10,
        g_min=0.2,
        g_max=3,
        recovery_min_per_s=0.01,
        recovery_max_per_s=10,
        w_min=0.1,
        t_x_s=60,
        epsilon_x=0.01,
        k_max=0.5,
        caretaker=dict(person_id="u-star", attachment_prior=60),
        r_capacity=10,
        protected_candidates=1,
        relation_concepts=["nemu.relation.closeness", "nemu.relation.trust"],
        registry_version="P5-71-v2-registry",
        theta_b=[0] * 4,
        theta_sigma=[0.25] * 4,
    )
    return c, f, g


def ledger() -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []

    def add(
        id_: str,
        family: str,
        world: Any,
        branch: Any,
        n: int,
        refs: list[str],
        mutants: list[str],
        dut: bool = True,
        reason: str = "",
    ) -> None:
        for version in refs + mutants:
            for layer in ("DUT", "independent_research"):
                why = reason if not dut else ""
                if version not in (
                    "old",
                    "A",
                    "A1",
                    "oldC-oldF",
                    "A-oldF",
                    "oldC-A1",
                    "A-A1",
                    "G",
                ):
                    why = "research-only reference or mutation; no production law/override"
                if layer == "independent_research":
                    why = ""
                rows.append(
                    dict(
                        id=id_,
                        family=family,
                        layer=layer,
                        world=world,
                        branch=branch,
                        version=version,
                        N=n,
                        planned_status="not_run" if why else "pending_budget_gate",
                        reason=why,
                        sum_N=0 if why else n,
                        prefix_recompute=0 if why else n * (n + 1) // 2,
                        recipe=f"PREREGISTRATION.md@{PREREG}:{id_}",
                    )
                )

    c_refs = ["old", "A", "B"]
    c_mut = {
        1: ["fixed-nb"],
        2: ["sign-screen", "x-as-a"],
        3: ["fixed-nb", "time-reset"],
        4: ["pull-to-birth"],
        5: ["time-learning", "missing-as-zero"],
        6: ["duplicate-mass"],
        7: ["subtract-only", "unknown-as-zero"],
        9: ["fixed-nb", "hide-prefix-error"],
        10: ["underflow-fault", "flush-subnormal"],
        11: ["old-b-segment", "wall-clock"],
        12: ["clip-continue", "alarm-reset", "old-nb-bound"],
    }
    for row, n in [(1, 128), (2, 128), (3, 256), (4, 256), (9, 1280)]:
        for p, rho, b in itertools.product(
            (1, 2, 16), ("dynamic", "1/2", "7/8"), range(1 if row == 9 else 2)
        ):
            add(
                f"07-{row:02}",
                "C",
                dict(p0=p, rho=rho),
                b,
                n,
                c_refs,
                c_mut[row],
                rho == "dynamic",
                "无固定 ρ 公开入口",
            )
    for row, w, branches, n in [
        (5, 6, 1, 32),
        (6, 2, 3, 32),
        (7, 2, 4, 32),
        (11, 4, 1, 64),
        (12, 2, 5, 4),
    ]:
        for world, b in itertools.product(range(w), range(branches)):
            add(f"07-{row:02}", "C", world, b, n, c_refs, c_mut[row])
    add("07-08", "C", 0, 0, 0, [], [], False, "P5-76 feedback")
    for k, n in enumerate([1074, 1, 1, 1, 1, 1]):
        add(
            "07-10",
            "C" if k < 2 else "arithmetic",
            k,
            0,
            n,
            c_refs,
            c_mut[10],
            False,
            "无固定 ρ 公开入口",
        )
    f_mut = {
        1: ["text-root", "action-id-root"],
        2: ["double-original-recall"],
        3: ["retransmit-as-revisit", "reject-revisit"],
        4: ["text-new-slot", "kappa-one"],
        5: ["derived-new-root", "derived-new-mass"],
        6: ["add-not-replace", "sort-first-by-name"],
        7: ["unverified-accepted", "reject-result", "result-learns-C"],
        8: ["ban-revisit"],
        9: ["unknown-new", "internal-revisit", "reject-internal-update"],
        10: ["candidate-mass", "reject-backfill"],
        11: ["clear-consumption", "guess-decomposition", "evict-nonzero"],
        12: ["target-difference", "net-x-only"],
        13: ["A0"],
        14: ["read-reanchor", "sum-h-x"],
        15: ["proportional"],
        16: ["force-equal", "loss-to-background"],
        17: ["zero-h", "zero-x"],
        18: ["x-only-contraction"],
        19: ["wrong-theta", "wrong-R", "wrong-interpretation"],
    }
    f_shapes = {
        1: (1, 32),
        2: (1, 3),
        3: (2, 3),
        4: (2, 128),
        5: (1, 32),
        6: (2, 30),
        7: (8, 4),
        8: (2, 2),
        9: (6, 3),
        10: (4, 4),
        11: (7, 8),
        12: (3, 8),
        13: (1, 2),
        14: (4, 2),
        15: (1, 4),
        16: (1, 3),
        17: (2, 5),
        18: (243, 1),
        19: (1, 7),
    }
    for row, (variants, n) in f_shapes.items():
        for s, j, k in itertools.product((-1, 1), range(4), range(variants)):
            for state_side in range(2 if row == 18 else 1):
                add(
                    f"09-{row:02}",
                    "F",
                    dict(sign=s, coordinate=j, variant=k),
                    state_side,
                    n,
                    ["old", "A1", "A2", "B"],
                    f_mut[row],
                )
    for s, j, b in itertools.product((-1, 1), range(4), range(4)):
        # Four F references share exactly one G trajectory, not four G laws.
        add("09-10-G", "G", dict(sign=s, coordinate=j), b, 5, ["G"], [])
    combos = ["oldC-oldF", "A-oldF", "oldC-A1", "A-A1", "B-A1", "A-A2", "A-B"]
    cmuts = [
        ["recompute-targets", "C-clears-x"],
        ["revisit-current", "new-old-state"],
        ["x-as-a", "internal-C-mass"],
        ["alarm-clears", "clip-continue"],
    ]
    for row, branches, n in [(1, 1, 32), (2, 1, 16), (3, 3, 64), (4, 5, 8)]:
        for s, j, b in itertools.product((-1, 1), range(4), range(branches)):
            add(
                f"C-{row:02}",
                "CF",
                dict(sign=s, coordinate=j),
                b,
                n,
                combos,
                cmuts[row - 1],
            )
    for id_, w in [("07-12", 2), ("C-04", 8)]:
        for world, coord, point, version in itertools.product(
            range(w),
            range(12),
            ("equal", "next_up", "nan", "inf", "-inf"),
            ("normal", "clip", "ignore-nonfinite", "greater-equal"),
        ):
            rows.append(
                dict(
                    id=id_ + "-model",
                    family="candidate",
                    layer="sensitivity_model",
                    world=world,
                    branch=dict(coordinate=coord, point=point),
                    version=version,
                    N=1,
                    sum_N=1,
                    prefix_recompute=1,
                    planned_status="pending_budget_gate",
                    reason="",
                    recipe=f"PREREGISTRATION.md@{PREREG}:§4",
                )
            )
        rows.append(
            dict(
                id=id_ + "-internal-DUT",
                family="candidate",
                layer="DUT",
                world="all",
                branch="all",
                version="A",
                N=60 * w,
                sum_N=0,
                prefix_recompute=0,
                planned_status="not_run",
                reason="无内部候选破坏/等界检查公开入口",
                recipe=f"PREREGISTRATION.md@{PREREG}:§4",
            )
        )
    for name, id_, family, w, n in [
        ("F-off", "09-02", "F", 8, 3),
        ("F-off", "09-07", "F", 16, 4),
        ("C-frozen", "07-02", "C", 6, 128),
        ("C-frozen", "C-01", "CF", 8, 32),
        ("G-frozen", "09-10-G", "G", 8, 5),
        ("x-zero", "09-02", "F", 8, 3),
        ("x-zero", "09-13", "F", 8, 2),
    ]:
        for world in range(w):
            # Freeze/off are adapter controls, not new production laws.
            add("NC-" + name + "-" + id_, family, world, 0, n, [], [name])
            rows[-2].update(
                planned_status="pending_budget_gate",
                reason="",
                sum_N=n,
                prefix_recompute=n * (n + 1) // 2,
            )
    # Explicit wiring-only mutations remain visible, never research substitution.
    for entry in rows:
        if entry["version"] == "clear-consumption":
            entry.update(
                planned_status="not_run",
                reason="P5-72/75 consumption wiring",
                sum_N=0,
                prefix_recompute=0,
            )
    return rows


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("output", type=Path)
    args = ap.parse_args()
    out = args.output
    out.mkdir(parents=True, exist_ok=False)
    c, f, g = resources()
    for name, resource in [
        ("C", c),
        ("F", f),
        ("G", g),
        (
            "C-alarm",
            dict(c, version="P5-71-v2-C-alarm", jump_notice_bits="0000000000000001"),
        ),
    ]:
        if name == "C-alarm":
            del resource["jump_notice"]
        write(out / (name + ".json"), resource)
    cases = []
    for b, v in itertools.product((-0.25, 0.25), ("old", "A")):
        cases.append(
            dict(
                id=f"07-01-p2-b{b}-{v}",
                family="C",
                p0=2,
                b0=b,
                version=v,
                events=[
                    dict(
                        id=f"probe-c-{i}",
                        ordinal=i,
                        time_ns=i * 10**9,
                        a=0.75 if i % 29 < 14 else -0.5,
                    )
                    for i in range(1, 33)
                ],
            )
        )
    for version in ("old", "A1"):
        cases.append(
            dict(
                id="09-06-plus-j0-123-" + version,
                family="F",
                version=version,
                events=[
                    dict(
                        id=f"probe-f-{i}",
                        ordinal=i,
                        time_ns=i * 10**9,
                        root="r" + str((i - 1) % 3 + 1),
                        first_position=(i - 1) % 3 + 1,
                        target=(0.25, 0.5, 0.75)[(i - 1) % 3],
                    )
                    for i in range(1, 31)
                ],
            )
        )
    write(out / "probe-inputs.json", cases)
    plan = ledger()
    write(out / "full-plan.json", plan)
    totals: dict[str, dict[str, int]] = {}
    for row in plan:
        key = row["layer"] + "/" + row["family"]
        t = totals.setdefault(
            key, dict(rows=0, sum_N=0, prefix_recompute=0, not_run_rows=0)
        )
        t["rows"] += 1
        t["sum_N"] += row["sum_N"]
        t["prefix_recompute"] += row["prefix_recompute"]
        t["not_run_rows"] += row["planned_status"] == "not_run"
    write(out / "plan-summary.json", totals)
    write(
        out / "preparation.json",
        dict(
            base=BASE,
            prereg=PREREG,
            mode="materialization_only",
            files={p.name: binding(p) for p in sorted(out.glob("*.json"))},
        ),
    )


if __name__ == "__main__":
    main()
