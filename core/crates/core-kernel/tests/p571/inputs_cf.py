"""Frozen v2 §6 interaction schedules; C and F are separate named substeps."""

from typing import Any


def cf_case(row: dict[str, Any]) -> dict[str, Any]:
    s, j = row["world"]["sign"], row["world"]["coordinate"]
    branch, num = row["branch"], int(row["id"][-2:])
    case: dict[str, Any] = dict(
        id=row["id"],
        version=row["version"],
        c_resource="C-alarm" if num == 4 and branch == 1 else "C",
        f_resource="F-interaction",
        p0=2,
        b0=0,
        rho="dynamic",
        events=[],
        coordinate=j,
        sign=s,
    )
    for i in range(1, row["N"] + 1):
        at = (7 if num == 2 and i > 7 else i) * 10**9
        c: dict[str, Any] | None = None
        f: dict[str, Any] | None = None
        if (
            (num == 1 and 2 <= i <= 16)
            or (num == 2 and 2 <= i <= 4)
            or num == 3
            or (num == 4 and i <= 4)
        ):
            a: float | str = (
                -s / 2
                if num == 1
                else s / 2
                if num == 2
                else (0.5 if i % 2 else -0.5)
                if num == 3
                else float(s)
            )
            if num == 4 and i == 2 and branch >= 2:
                a = ("nan", "inf", "-inf")[branch - 2]
            c = dict(
                id=f"{row['id']}-C{i}",
                ordinal=i,
                time_ns=at,
                stopped_ns=0,
                running_ns=at,
                stops=[],
                a=a,
                eligible=True,
                source="other_person",
                qualification_reason="qualified external observation",
            )
        op, root, target, mode = "", "r1", s / 2, "direct"
        if num == 1 and i in (1, 17):
            op = "new" if i == 1 else "revisit"
        elif num == 2 and (i == 1 or i >= 5):
            op = "new" if i in (1, 5) else "revisit" if i == 6 else "update"
            root, target, mode = "r2" if i == 5 else "r1", s / 4, "current_theta"
            if i == 6:
                mode = "recorded_target"
        elif num == 3 and (i == 1 or branch != 0):
            op = "new" if i == 1 or branch == 2 else "revisit"
            root = f"r{i}" if branch == 2 else "r1"
        elif num == 4 and i > 4:
            op, root, target = "new", f"r{i - 4}", s
        if op:
            vector = [0.0] * 4
            vector[j] = target
            f = dict(
                id=f"{row['id']}-F{i}",
                ordinal=i,
                time_ns=at,
                op=op,
                root=root,
                kind="record",
                target=vector,
                target_mode=mode,
                a=vector if num == 2 else None,
                mu=None,
                w=1,
            )
        case["events"].append(
            dict(
                ordinal=i,
                time_ns=at,
                c=c,
                f=f,
                substeps=["project_old_theta", "C", "F", "record_new_theta"],
            )
        )
    return case
