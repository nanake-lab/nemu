"""Frozen measurement mutations and paired real-arithmetic witnesses."""

from typing import Any
from arithmetic import Q
from c_oracle import first_difference


def distance(case: dict[str, Any]) -> dict[str, Any]:
    pair = case["distance_pair"]
    v = Q(pair["target"])
    before = []
    after = []
    for side in (1, 2):
        x, h = Q(pair[f"x{side}"]), Q(pair[f"h{side}"])
        total = x + v - h
        if -1 < total < 1:
            xp, hp = total, v
        else:
            xp = Q(1 if total >= 1 else -1)
            hp = max(min(h, v), min(max(h, v), h + xp - x))
        before.append((x, h))
        after.append((xp, hp))
    old_x = abs(before[0][0] - before[1][0])
    new_x = abs(after[0][0] - after[1][0])
    old_d = old_x + abs(before[0][1] - before[1][1])
    new_d = new_x + abs(after[0][1] - after[1][1])
    return dict(
        real_before=str(old_d),
        real_after=str(new_d),
        extended_nonexpansive=new_d <= old_d,
        x_before=str(old_x),
        x_after=str(new_x),
        x_only_contraction=new_x <= old_x,
        same_slot=before[0][1] == before[1][1],
        actual_scope="A1 exact scalar paired inputs; no B6 reachability premise",
    )


def measurement_mutation(
    version: str,
    prefix: int,
    n: int,
    case: dict[str, Any],
    baseline: dict[str, Any] | None,
    current: dict[str, Any],
) -> tuple[str, Any, Any] | None:
    if prefix == 0:
        return None
    if version == "hide-prefix-error" and prefix < n:
        return (
            "required-prefix-error-curve",
            f"prefix {prefix}",
            "mutant publishes only final limit",
        )
    if version == "net-x-only" and baseline:
        state = baseline["state"]
        if state.get("h") or state.get("targets"):
            return (
                "mandatory-slots-and-target-measurement",
                list(state.get("h", {})),
                "mutant retains x only",
            )
    if version == "x-only-contraction":
        witness = distance(case)
        if not witness["x_only_contraction"]:
            return ("x-only-contraction", witness["x_before"], witness["x_after"])
    if version == "old-nb-bound" and baseline:
        return first_difference(
            baseline["audit"].get("bounds"),
            current["audit"].get("bounds"),
            "bounds-new-prior-term",
        )
    return None
