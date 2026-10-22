"""CPU-only timing bookkeeping; no model, CUDA, or output-text side effects."""
from __future__ import annotations

from collections.abc import Callable
import time


class TokenClock:
    """Pinned Transformers calls put(prompt) once, then put(next_tokens.cpu()) per step."""

    def __init__(self, started: float, now: Callable[[], float] = time.perf_counter) -> None:
        self.started = started
        self.now = now
        self.prompt_seen = False
        self.first_ms: float | None = None
        self.last_ms: float | None = None
        self.generated = 0

    def put(self, value: object) -> None:
        if not self.prompt_seen:
            self.prompt_seen = True
            return
        elapsed = (self.now() - self.started) * 1000
        if self.first_ms is None:
            self.first_ms = elapsed
        self.last_ms = elapsed
        self.generated += 1

    def end(self) -> None:
        return


def decode_rate(tokens: int, first_ms: float | None, last_ms: float | None) -> float | None:
    """Exclude the first token and prefill; EOS remains part of generated-token count."""
    if tokens <= 1 or first_ms is None or last_ms is None or last_ms <= first_ms:
        return None
    return (tokens - 1) * 1000 / (last_ms - first_ms)


def arm_order(index: int) -> tuple[str, str]:
    return ("unconstrained", "constrained") if index % 2 == 0 else ("constrained", "unconstrained")
