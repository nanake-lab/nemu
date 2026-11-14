"""Independent F analytic readouts; expectations never use DUT state.

Intervals are derived from exact operands and directed binary64 endpoints.
The elementary-function value is conditional, not a certified exp enclosure.
"""

from copy import deepcopy
from typing import Any

from arithmetic import Q, outward, rounded as r
from f_research import FResearch

Interval = tuple[Q, Q]


def enclosure(lo: Q, hi: Q) -> Interval:
    return Q(outward(lo, False)), Q(outward(hi, True))


def add(a: Interval, b: Interval) -> Interval:
    return enclosure(a[0] + b[0], a[1] + b[1])


def sub(a: Interval, b: Interval) -> Interval:
    return enclosure(a[0] - b[1], a[1] - b[0])


def mul(a: Interval, b: Interval) -> Interval:
    values = [x * y for x in a for y in b]
    return enclosure(min(values), max(values))


def clamp(a: Interval, lo: Q = Q(-1), hi: Q = Q(1)) -> Interval:
    return max(lo, min(hi, a[0])), max(lo, min(hi, a[1]))


def point(a: Q) -> Interval:
    return a, a


def serialize(v: Any) -> Any:
    if isinstance(v, Q):
        return str(v)
    if isinstance(v, (tuple, list)):
        return [serialize(x) for x in v]
    if isinstance(v, dict):
        return {k: serialize(x) for k, x in v.items()}
    return v


class Analysis:
    def __init__(self, model: FResearch) -> None:
        self.real: dict[str, list[Interval]] = {}
        self.binary: dict[str, list[Interval]] = {}
        self.y: list[Q] | None = [
            x - sum((v[j] for v in model.h.values()), Q(0))
            for j, x in enumerate(model.x)
        ]
        self.error = [point(Q(0)) for _ in model.x]
        for root, values in model.h.items():
            self.real[root] = [(min(Q(0), x), max(Q(0), x)) for x in values]
            self.binary[root] = list(self.real[root])

    def observe(
        self,
        before: FResearch,
        after: FResearch,
        event: dict[str, Any] | None,
        projected: bool,
    ) -> dict[str, Any]:
        size = before.size
        local = [point(Q(0)) for _ in range(size)]
        px, ph = list(before.x), deepcopy(before.h)
        if projected:
            dt = r(Q(after.anchor - before.anchor) / 10**9)
            rates = before.theta["lambda"] + (
                [before.theta["lambda"][before.case["coordinate"]]] if size == 5 else []
            )
            rests = before.theta["b"] + ([Q(0)] if size == 5 else [])
            px, ph = before.projection(after.anchor)
            for j, (rate, rest) in enumerate(zip(rates, rests)):
                pull = -before.fn("expm1", -r(rate * dt))
                e = 1 - pull
                xi = clamp(
                    add(
                        point(before.x[j]),
                        mul(point(pull), sub(point(rest), point(before.x[j]))),
                    )
                )
                dx = sub(xi, point(rest + e * (before.x[j] - rest)))
                dh = point(Q(0))
                for values in before.h.values():
                    hi = add(point(values[j]), mul(point(pull), point(-values[j])))
                    dh = add(dh, sub(hi, point(e * values[j])))
                local[j] = sub(dx, dh)
                self.error[j] = add(mul(point(e), self.error[j]), local[j])
                if self.y is not None:
                    self.y[j] = rest + e * (self.y[j] - rest)
        op = event["op"] if event else "none"
        if op == "migrate":
            self.real.clear()
            self.binary.clear()
            self.y = None
        elif op == "remove_zero":
            # History intervals belong to the basis, not its removable zero slot.
            pass
        elif op == "merge" and after.base not in ("old", "B"):
            assert event is not None
            members = sorted(
                event["members"], key=lambda k: before.history[k]["ordinal"]
            )
            real = [point(Q(0)) for _ in range(size)]
            binary = list(real)
            for root in members:
                for j in range(size):
                    a, b = real[j], self.real.get(root, [point(Q(0))] * size)[j]
                    real[j] = a[0] + b[0], a[1] + b[1]
                    a, b = binary[j], self.binary.get(root, [point(Q(0))] * size)[j]
                    binary[j] = r(a[0] + b[0]), r(a[1] + b[1])
            for root in members:
                self.real.pop(root, None)
                self.binary.pop(root, None)
            self.real[members[0]], self.binary[members[0]] = real, binary
            for j in range(size):
                # Merge's finite ordered sum has a separately measured rounding residual.
                ideal = sum((before.h.get(k, [Q(0)] * size)[j] for k in members), Q(0))
                actual = after.h[members[0]][j]
                self.error[j] = add(self.error[j], point(ideal - actual))
        elif (
            after.last.get("applied")
            and event is not None
            and after.base not in ("old", "B")
        ):
            root = event["root"]
            target = after.targets[root]
            for table in (self.real, self.binary):
                interval = table.setdefault(root, [point(Q(0)) for _ in range(size)])
                for j, value in enumerate(target):
                    interval[j] = min(interval[j][0], value), max(interval[j][1], value)
            for j, v in enumerate(target):
                h = ph.get(root, [Q(0)] * size)[j]
                xi = clamp(add(point(px[j]), sub(point(v), point(h))))
                total = px[j] + v - h
                real_x = max(Q(-1), min(Q(1), total))
                if after.base == "A2" or after.mutant in (
                    "A0",
                    "loss-to-background",
                    "force-equal",
                ):
                    hi = point(v)
                    ideal_h = v
                    if self.y is not None:
                        self.y[j] += real_x - px[j] - (v - h)
                else:
                    ideal_h = h + real_x - px[j]
                    # Include both sides when rounding can cross the strict branch boundary.
                    candidates = [point(v)]
                    for wall in (Q(-1), Q(1)):
                        candidates.append(
                            clamp(
                                add(point(h), sub(point(wall), point(px[j]))),
                                min(h, v),
                                max(h, v),
                            )
                        )
                    total_i = add(point(px[j]), sub(point(v), point(h)))
                    if total_i[0] > -1 and total_i[1] < 1:
                        hi = candidates[0]
                    elif total_i[0] >= 1:
                        hi = candidates[2]
                    elif total_i[1] <= -1:
                        hi = candidates[1]
                    else:
                        hi = (
                            min(vv[0] for vv in candidates),
                            max(vv[1] for vv in candidates),
                        )
                residual = sub(sub(xi, point(real_x)), sub(hi, point(ideal_h)))
                local[j] = add(local[j], residual)
                self.error[j] = add(self.error[j], residual)
        observed_y = [
            x - sum((v[j] for v in after.h.values()), Q(0))
            for j, x in enumerate(after.x)
        ]
        residuals = (
            [a - b for a, b in zip(observed_y, self.y)] if self.y is not None else None
        )
        bounds_ok = all(
            self.binary[k][j][0] <= value <= self.binary[k][j][1]
            for k, values in after.h.items()
            for j, value in enumerate(values)
        )
        error_ok = (
            all(lo <= value <= hi for value, (lo, hi) in zip(residuals, self.error))
            if residuals is not None
            else None
        )
        return dict(
            serialize(
                dict(
                    history_real=self.real,
                    history_binary=self.binary,
                    history_contains_h=bounds_ok,
                    independent_background=self.y,
                    decomposition_residual=residuals,
                    local_error=local,
                    cumulative_error=self.error,
                    residual_enclosed=error_ok,
                    transcendental_error="not_run",
                    scope="conditional on locked libm pull; not ideal continuous-time enclosure",
                )
            )
        )
