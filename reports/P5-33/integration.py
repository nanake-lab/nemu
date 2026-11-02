"""Research adapters around UNMODIFIED ADR 0019 scalar/event reference code."""
from __future__ import annotations

import copy
import sys
from dataclasses import dataclass, field, fields
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "echo-core-v2-draft"))
from law_v2_events import Engine as ReferenceEngine  # noqa: E402
from law_v2_events import Event as Event  # noqa: E402
from law_v2_model import Tendency as Reference  # noqa: E402
from law_v2_model import bits  # noqa: E402
from detectors import Detector, make_detector  # noqa: E402


@dataclass
class Controlled(Reference):
    detector: Detector = field(default_factory=lambda: make_detector("flips"))

    def add(self, w: float, s: float, mu: float = 0.0, projection: float | None = None) -> float:
        staged = copy.copy(self.detector)  # observe replaces lists; never mutates existing nodes
        h = staged.observe(s)
        old = self.fixed_rho
        self.fixed_rho = 1.0-h
        try:
            q = super().add(w, s, mu, projection)
        except ValueError:
            self.fixed_rho = old
            raise
        self.detector = staged
        return q

    def full_state(self) -> tuple[str, object]:
        return bits(self), self.detector.state()


@dataclass
class ResearchEngine(ReferenceEngine):
    method: str = "flips"

    def ensure(self, name: str) -> None:
        if name not in self.tendencies:
            self.tendencies[name] = Controlled(3.0, 1.0, prior=self.prior,
                                               detector=make_detector(self.method))
            self.allocations[name] = []

    def replay_tendency(self, name: str) -> None:
        # Keep historical q exactly; reconstruct sign-only detector from the same records.
        super().replay_tendency(name)
        scalar = self.tendencies[name]
        values: dict[str, Any] = {f.name: getattr(scalar, f.name) for f in fields(Reference)}
        tracker = make_detector(self.method)
        h = 0.1
        for row in self.allocations[name]:
            h = tracker.observe(row.support)
        self.tendencies[name] = Controlled(**values, detector=tracker)
        self.tendencies[name].fixed_rho = 1.0-h if self.allocations[name] else None

    def full_state(self) -> object:
        return (self.snapshot(), [(k, self.tracker(k).state()) for k in sorted(self.tendencies)])

    def tracker(self, name: str) -> Detector:
        model = self.tendencies[name]
        assert isinstance(model, Controlled)
        return model.detector
