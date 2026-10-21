"""Finite reference models; parameters are declared experiment choices."""
from typing import Protocol


class Predictor(Protocol):
    def predict(self) -> float: ...
    def update(self, y: int) -> None: ...


class Mean:
    def __init__(self, window: int | None = None) -> None:
        self.window = window
        self.history: list[int] = []
        self.success = 0

    def predict(self) -> float:
        return (1 + self.success)/(2 + len(self.history))

    def update(self, y: int) -> None:
        self.history.append(y)
        self.success += y
        if self.window is not None and len(self.history) > self.window:
            self.success -= self.history.pop(0)


class Change:
    def __init__(self, hazard: float) -> None:
        self.hazard = hazard
        self.runs = [(1.0, 1.0, 1.0)]

    def predict(self) -> float:
        return self.hazard*.5 + (1-self.hazard)*sum(w*a/(a+b) for w, a, b in self.runs)

    def update(self, y: int) -> None:
        h = self.hazard
        new = [(h*.5, 1.0+y, 2.0-y)]
        for w, a, b in self.runs:
            new.append(((1-h)*w*(a if y else b)/(a+b), a+y, b+1-y))
        total = sum(w for w, _, _ in new)
        self.runs = [(w/total, a, b) for w, a, b in new]


class Adaptive:
    def __init__(self) -> None:
        self.hazards = (0.0, .002, .01, .05)
        self.models: list[Predictor] = [Mean()] + [Change(h) for h in self.hazards[1:]]
        self.weights = [.25]*4

    def predict(self) -> float:
        return sum(w*m.predict() for w, m in zip(self.weights, self.models))

    def update(self, y: int) -> None:
        weights = []
        for w, model in zip(self.weights, self.models):
            p = model.predict()
            weights.append(w*(p if y else 1-p))
        total = sum(weights)
        self.weights = [w/total for w in weights]
        for model in self.models:
            model.update(y)

    def mean_hazard(self) -> float:
        return sum(w*h for w, h in zip(self.weights, self.hazards))


def models() -> dict[str, Predictor]:
    return {"cumulative": Mean(), "window16": Mean(16), "window64": Mean(64),
            "fixed_change": Change(.01), "adaptive": Adaptive()}
