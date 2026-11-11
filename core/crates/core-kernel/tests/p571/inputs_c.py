"""Frozen v2 §4 histories. Materialization only; no recurrence or expected values."""

from typing import Any


def c_case(row: dict[str, Any]) -> dict[str, Any]:
    id_, world, branch = row["id"], row["world"], row["branch"]
    p0 = world["p0"] if isinstance(world, dict) else 2
    rho = world["rho"] if isinstance(world, dict) else "dynamic"
    b0 = (-0.25, 0.25)[branch] if id_ == "07-01" else 0.0
    sign = (-1, 1)[world] if id_ in ("07-06", "07-07", "07-12") else 1
    case: dict[str, Any] = dict(
        id=id_, p0=p0, b0=b0, rho=rho, version=row["version"], events=[], reads_ns=[]
    )
    if id_ == "07-10":
        case.update(p0=1, b0=1, rho="1/2")
        if world:
            # All numbers are serialized as exact bit patterns or rational
            # strings. These are inputs, never expected rounded answers.
            case["initial"] = [
                dict(P_bits="0000000000000002", S=2, U=1, a=-0.5, rho="1/2"),
                dict(P_bits="0000000000000001", rho="3/4"),
                dict(P_bits="0000000000000001", rho="1/2"),
                dict(P_bits="0000000000000002", rho="3/4"),
                dict(
                    P_bits="0010000000000000", rho="9007199254740991/9007199254740992"
                ),
            ][world - 1]
            case["rho"] = case["initial"]["rho"]
    for i in range(1, row["N"] + 1):
        a: float | str = 0.5
        source, eligible, reason = (
            "other_person",
            True,
            "qualified external observation",
        )
        time_ns, stopped_ns = i * 10**9, 0
        if id_ == "07-01":
            a = 0.75 if i % 29 < 14 else -0.5
        elif id_ == "07-02":
            a = (-0.75, 0.75)[branch]
        elif id_ in ("07-03", "07-04"):
            if id_ == "07-03":
                a = -0.5 if branch == 1 and 65 <= i <= 80 else 0.5
            else:
                a = -0.5 if i >= 65 and (branch == 0 or i <= 80) else 0.5
        elif id_ == "07-05" and i > 8:
            source = [
                "none",
                "self_action",
                "program_result",
                "self_narrative",
                "desktop",
                "missing_A",
            ][world]
            eligible, reason = False, "not an eligible external reaction"
        elif id_ == "07-06":
            a = sign / 2
            eligible = branch == 2 or (i - 1) % 4 == 0
            reason = (
                "independent contact" if eligible else "same contact already admitted"
            )
        elif id_ == "07-07":
            a = sign / 2 if i <= 16 else -sign / 2
            if i == 8 and branch in (1, 3):
                eligible = False
                reason = "never valid" if branch == 1 else "missing A"
                if branch == 3:
                    source = "missing_A"
        elif id_ == "07-09":
            a = 0.5 if i <= 1024 else -0.5
        elif id_ == "07-10":
            a = 0.5 if world == 0 and i <= 1073 else -0.5
        elif id_ == "07-11":
            a = -0.5 if 17 <= i <= 32 else 0.5
            if world == 1:
                time_ns = 0
            elif world == 2 and i >= 17:
                time_ns, stopped_ns = (i + 16) * 10**9, 16 * 10**9
        elif id_ == "07-12":
            a = sign
            if i == 2 and branch >= 2:
                a = ("nan", "inf", "-inf")[branch - 2]
        event: dict[str, Any] = dict(
            id=f"{id_}-e{i}",
            ordinal=i,
            time_ns=time_ns,
            stopped_ns=stopped_ns,
            running_ns=time_ns - stopped_ns,
            a=a,
            source=source,
            eligible=eligible,
            qualification_reason=reason,
            stops=(
                [dict(start_ns=16 * 10**9, end_ns=32 * 10**9)] if stopped_ns else []
            ),
        )
        if id_ == "07-06":
            contact = i if branch == 2 else (i - 1) // 4 + 1
            event.update(
                contact=f"contact-{contact}",
                root=f"contact-{contact}",
                stable_source_key=f"source-{i if branch == 1 else contact}",
            )
        if id_ == "07-07" and branch == 0 and i == 17:
            event["invalidate_prior_ordinals"] = [8]
        if id_ == "07-07" and branch == 2 and i == 17:
            event["recants_prior_ordinal"] = 8
        if source == "missing_A":
            event["a"] = None
        case["events"].append(event)
    if id_ == "07-05":
        case["reads_ns"] = [0, 10**9, 86400 * 10**9, 365 * 86400 * 10**9]
        case["read_time_origin"] = "offset_from_last_committed_anchor"
        case["read_initial_x"] = [0.0] * 4
        case["read_scope"] = (
            "pure Phi from the explicit F genesis x0 at the last C anchor; no runtime wiring"
        )
    if id_ == "07-11":
        case["clock_branch"] = (
            "one_second",
            "zero_interval",
            "downtime",
            "open_final_segment",
        )[world]
        case["close_final_segment"] = False
    case["resource"] = "C-alarm" if id_ == "07-12" and branch == 1 else "C"
    return case
