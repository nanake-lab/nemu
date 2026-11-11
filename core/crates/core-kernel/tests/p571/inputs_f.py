"""Frozen v2 §5 scripts, with explicit governance, times and historical roots."""

from itertools import product
from typing import Any


def f_case(row: dict[str, Any]) -> dict[str, Any]:
    world = row["world"]
    s, j, variant = world["sign"], world["coordinate"], world["variant"]
    num = int(row["id"][-2:])
    case: dict[str, Any] = dict(
        id=row["id"],
        version=row["version"],
        coordinate=j,
        sign=s,
        resource="F",
        events=[],
        reads_ns=[],
        theta=dict(b=[0.0] * 4, g=[1.0] * 4, lambda_bits=["3fe62e42fefa39ef"] * 4),
    )

    def event(
        i: int,
        op: str,
        root: str,
        target: float = 0.5,
        at: int = 0,
        kind: str = "record",
        **extra: Any,
    ) -> dict[str, Any]:
        a = [0.0] * 4
        a[j] = s * target
        return dict(
            id=f"{row['id']}-e{i}",
            ordinal=i,
            op=op,
            root=root,
            target=a,
            time_ns=at,
            kind=kind,
            qualification="fixed synthetic admission table",
            **extra,
        )

    def add(
        op: str,
        root: str = "r1",
        target: float = 0.5,
        at: int | None = None,
        kind: str = "record",
        **extra: Any,
    ) -> None:
        if at is None:
            at = (len(case["events"]) + 1) * 10**9
        case["events"].append(
            event(len(case["events"]) + 1, op, root, target, at, kind, **extra)
        )

    if num in (1, 2, 5):
        for i in range(row["N"]):
            add(
                "new" if i == 0 else "revisit",
                at=(i // 4) * 10**9 if num == 5 else 0,
                surface=("original", "zh paraphrase", "en paraphrase", "ja paraphrase")[
                    i % 4
                ],
                derived="original"
                if i == 0
                else ("retrieval", "summary", "echo")[(i - 1) % 3],
            )
    elif num == 3:
        add("new", at=0)
        add(
            "noop" if variant == 0 else "revisit",
            at=10**9,
            admission="retransmit" if variant == 0 else "new lawful revisit",
        )
        add("noop", at=2 * 10**9)
        case["events"][0].update(record_id="original-record", stage="F_in")
        case["events"][1].update(
            record_id="original-record" if variant == 0 else "new-revisit-record",
            stage="F_in",
            referenced_original="original-record",
        )
    elif num == 4:
        for i in range(128):
            add(
                "new" if i == 0 else "update" if variant == 0 else "noop",
                "internal",
                (0.25, 0.5, 0.75, 0.5)[i % 4],
                (i + 1) * 10**9,
                "internal",
                surface=f"internal narrative {i}",
            )
    elif num == 6:
        order = (1, 2, 3) if variant == 0 else (3, 2, 1)
        for i in range(30):
            root_index = order[i % 3]
            add(
                "new" if i < 3 else "revisit",
                f"r{root_index}",
                root_index / 4,
                (i + 1) * 10**9,
                first_position=order.index(root_index) + 1,
            )
    elif num == 7:
        governance, target_sign = divmod(variant, 2)
        add("new")
        for i in range(3):
            add(
                "noop"
                if governance == 1
                else "new"
                if i == 0 and governance == 0
                else "update"
                if i == 0
                else "revisit",
                "result" if governance == 0 else "r1",
                (-0.5, 0.5)[target_sign],
                (i + 2) * 10**9,
                governance=(
                    "verified_result",
                    "invalid_certificate",
                    "condition_change",
                    "counterevidence",
                )[governance],
                source="action_result" if governance == 0 else "fixed_governance",
            )
    elif num == 8:
        add("new")
        add("revisit", at=(1 + (1, 1024)[variant]) * 10**9)
    elif num == 9:
        kind_index, role = divmod(variant, 2)
        kind = "record" if kind_index == 2 else "internal"
        root = "verified-deduction" if kind_index == 2 else "internal"
        case["scenario_kind"] = (
            "internal_branch",
            "unrooted_creativity",
            "verified_deduction",
        )[kind_index]
        add("new", root, 0.25, kind=kind)
        add("update", root, kind=kind)
        add(
            "revisit" if role == 0 else "update",
            root,
            kind=kind,
            expect_rejection=kind == "internal" and role == 0,
        )
        case["novelty_witness"] = dict(
            observation="identical unverified creative text",
            compatible_truths=["new fact", "old fact"],
        )
    elif num == 10:
        for i in range(4):
            add(
                "new"
                if variant == 3 and i == 0
                else "revisit"
                if variant == 3
                else "noop",
                "old-evidence",
                admission=("candidate", "repeat", "alias", "verified_backfill")[
                    variant
                ],
            )
    elif num == 11:
        if variant == 0:
            add("new", target=0.25)
            add("invalidate")
            add("revisit", active=False)
        elif variant in (1, 6):
            first, second = (0.25, 0.5) if variant == 1 else (0.1, 0.2)
            add("new", "r1", first)
            add("new", "r2", second)
            add("merge", members=["r1", "r2"])
            add("revisit", target=first)
        elif variant == 2:
            add("new", target=0.25)
            add("update", target=0)
            add("remove_zero")
            add("update", target=0.5)
        elif variant == 3:
            add("new", target=0.25)
            add("remove_zero", require_retention=True)
            add("revisit", target=0.25)
        elif variant == 4:
            add("new", target=0.25)
            add("roundtrip", clear_adapter_cache=True)
            add("revisit", target=0.25)
        else:
            add("new", target=0.25)
            add("migrate", old_decomposition="unobserved")
            add("update", target=0.5)
            case["decomposition_witness"] = dict(
                x=s / 4,
                compatible_hidden_states=[
                    dict(h={}, y=s / 4),
                    dict(h={"r1": s / 2}, y=-s / 4),
                ],
            )
    elif num == 12:
        for i, (op, root, value) in enumerate(
            [
                ("new", "r1", 1),
                ("new", "r2", -1),
                ("new", "r3", 1),
                ("revisit", "r2", -1),
                ("revisit", "r1", 1),
                ("revisit", "r3", 1),
                ("update", "r2", 0),
                ("update", "r1", 0),
            ]
        ):
            add(op, root, value, i * variant * 10**9)
    elif num in (13, 15, 16, 17):
        target = 0.75 if num == 16 else 1.0
        add("new", "r1", target, 0)
        add("new", "r2", -1.0 if num == 17 and variant == 1 else target, 0)
        if num == 13:
            case["reads_ns"] = [0, 10**9, 2 * 10**9, 4 * 10**9]
        elif num == 15:
            add("revisit", "r1", target, 0)
            add("revisit", "r1", target, 0)
        elif num == 16:
            add("revisit", "r2", target, 10**9)
        elif variant == 0:
            add("update", "r1", 0, 10**9)
            add("update", "r2", 0, 10**9)
        else:
            add("new", "r3", 1, 0)
            add("update", "r2", 0, 0)
    elif num == 14:
        b_index, x_index = divmod(variant, 2)
        case["theta"]["b"] = [s / 4 if b_index else 0.0] * 4
        # LN2 times powers of two: exact exponent shifts, no new resource tuning.
        case["theta"]["lambda_bits"] = [
            "3fe62e42fefa39ef",
            "3ff62e42fefa39ef",
            "40062e42fefa39ef",
            "40162e42fefa39ef",
        ]
        add("new", "r1", (1.0, 0.5)[x_index], 0)
        add("new", "r2", 0, 0)
        case["reads_ns"] = [0, 125000000, 500000000, 10**9, 2 * 10**9, 8 * 10**9]
        case["events"][0]["readonly_after_ns"] = [0]
    elif num == 18:
        x1, h1, x2, h2, target = list(product((-1, 0, 1), repeat=5))[variant]
        x, h = (x1, h1) if row["branch"] == 0 else (x2, h2)
        case["initial"] = dict(x=s * x, h=s * h, root="r1", coordinate=j)
        case["distance_pair"] = dict(
            x1=s * x1, h1=s * h1, x2=s * x2, h2=s * h2, target=s * target
        )
        add("update", "r1", target)
    elif num == 19:
        for i, (op, revision, interpretation) in enumerate(
            [
                ("new", 0, "old"),
                ("update", 1, "old"),
                ("update", 2, "old"),
                ("reinterpret", 2, "new"),
                ("revisit", 2, "new"),
                ("update", 3, "new"),
                ("update", 3, "new"),
            ]
        ):
            add(
                op,
                target=0.25 if interpretation == "old" else -0.25,
                at=min(i + 1, 6) * 10**9,
                theta_revision=revision,
                effects_revision=revision,
                interpretation=interpretation,
                target_mode="evaluate_recorded_inputs",
                mu=0.5,
                w=1,
            )
            case["events"][-1].update(
                a=case["events"][-1]["target"],
                involved_people=[],
                governance_id=f"synthetic-governance-{i + 1}",
                interpretation_id=f"interpreter-{interpretation}",
            )
        case["historical_inputs"] = [
            dict(
                revision=k,
                theta_g=[(k + 1) / 4] * 4,
                effect_s=(k + 1) / 8,
                effect_r=0.5,
                effect_direction=[s if c == j else 0 for c in range(4)],
                mu=0.5,
                w=1,
            )
            for k in range(4)
        ]
    else:
        raise ValueError(f"unmaterialized F row {row['id']}")
    while len(case["events"]) < row["N"]:
        add("noop", at=case["events"][-1]["time_ns"])
    if len(case["events"]) != row["N"]:
        raise ValueError("frozen N mismatch")
    return case
