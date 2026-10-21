"""Research-only, typed evidence projection. Not a production IDL."""
from dataclasses import dataclass
from math import isfinite, log, pi, sqrt
from typing import Literal, Sequence

Kind = Literal["outcome", "user_preference", "self_description"]


@dataclass(frozen=True)
class Record:
    root: str
    revision: int
    order: int
    context: str
    action: str
    criterion: str
    owner: str = "nemu"
    kind: Kind = "outcome"
    outcome: int | None = None
    cost: float = 0.0
    retracted: bool = False

    def __post_init__(self) -> None:
        if not self.root or self.revision < 0 or self.order < 0:
            raise ValueError("invalid evidence identity")
        if self.outcome not in (0, 1, None):
            raise ValueError("outcome must be binary or unknown")
        if not isfinite(self.cost) or not 0 <= self.cost <= 1:
            raise ValueError("invalid measured cost")


@dataclass(frozen=True)
class Criterion:
    key: str
    owner: str = "nemu"
    cost_weight: float = 0.0

    def __post_init__(self) -> None:
        if not isfinite(self.cost_weight) or not 0 <= self.cost_weight <= 1:
            raise ValueError("invalid criterion parameter")


def canonical(history: Sequence[Record]) -> list[Record]:
    versions: dict[tuple[str, int], Record] = {}
    roots: dict[str, Record] = {}
    for row in history:
        key = (row.root, row.revision)
        if key in versions and versions[key] != row:
            raise ValueError("conflicting same-revision claim")
        versions[key] = row
        previous = roots.get(row.root)
        if previous is not None:
            fields = ("order", "context", "action", "criterion", "owner", "kind")
            if any(getattr(previous, f) != getattr(row, f) for f in fields):
                raise ValueError("revision cannot refresh identity or event order")
        if previous is None or row.revision > previous.revision:
            roots[row.root] = row
    return sorted(roots.values(), key=lambda row: (row.order, row.root))


def eligible(history: Sequence[Record], criterion: Criterion) -> list[Record]:
    return [r for r in canonical(history)
            if r.kind == "outcome" and not r.retracted
            and r.owner == criterion.owner and r.criterion == criterion.key]


@dataclass(frozen=True)
class Estimate:
    trials: int
    resolved: int
    forecast: float
    lower: float
    upper: float
    cost: float

    def value(self, criterion: Criterion) -> float:
        return self.forecast - criterion.cost_weight * self.cost


def estimate(rows: Sequence[Record], *, mar: bool = False,
             delta: float = .05, cells: int = 2) -> Estimate:
    """Bounds concern a stationary response mean, not a causal effect."""
    if not 0 < delta < 1 or cells < 1:
        raise ValueError("invalid diagnostic risk budget")
    n = len(rows)
    observed = [r.outcome for r in rows if r.outcome is not None]
    count, success = len(observed), sum(observed)
    forecast = (1 + success) / (2 + count)
    cost = sum(r.cost for r in rows) / n if n else 0.0
    denominator = count if mar else n
    if not denominator:
        return Estimate(n, count, forecast, 0.0, 1.0, cost)
    epsilon = sqrt(log(pi*pi*cells*denominator*denominator/(3*delta))/(2*denominator))
    low = success/denominator
    high = low if mar else (success+n-count)/n
    return Estimate(n, count, forecast, max(0.0, low-epsilon), min(1.0, high+epsilon), cost)


def justified_order(a: Estimate, b: Estimate) -> int | None:
    """An evidence label, never a prerequisite for updating or low-risk action."""
    if a.lower > b.upper:
        return 0
    if b.lower > a.upper:
        return 1
    return None


def predictions(history: Sequence[Record], criterion: Criterion,
                context: str, *, mar: bool = False) -> tuple[Estimate, Estimate]:
    rows = eligible(history, criterion)
    return tuple_estimates([estimate([r for r in rows if r.context == context and r.action == a], mar=mar)
                            for a in ("a", "b")])


def tuple_estimates(values: list[Estimate]) -> tuple[Estimate, Estimate]:
    return values[0], values[1]
