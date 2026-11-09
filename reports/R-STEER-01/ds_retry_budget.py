"""Serialized pre-POST reservations and a process-wide stop flag."""
from __future__ import annotations

import threading
import time
from pathlib import Path
from typing import Any

from ds_high_prepare import reserve
from ds_pilot import event


class Ledger:
    def __init__(self, plan: dict[str, Any], path: Path) -> None:
        self.plan, self.path = plan, path
        self.total = int(plan['prior_upper_nano_usd'])
        self.attempts = int(plan['prior_attempts'])
        self.lock = threading.Lock()
        self.stop = threading.Event()
        self.reason: str | None = None
        self.until = time.monotonic() + plan['remaining_wall_seconds']

    def halt(self, reason: str) -> None:
        with self.lock:
            if not self.stop.is_set():
                self.reason = reason
                event(self.path, {'event': 'stop_new_posts', 'reason': reason})
                self.stop.set()

    def book(self, identifier: str, attempt: int, body: bytes) -> int | None:
        with self.lock:
            if self.stop.is_set():
                return None
            held = reserve(body)
            if time.monotonic() >= self.until or self.total + held >= self.plan['soft_nano_usd']:
                self.reason = 'time_stop' if time.monotonic() >= self.until else 'budget_stop'
                event(self.path, {'event': 'stop_new_posts', 'reason': self.reason})
                self.stop.set()
                return None
            self.total += held
            self.attempts += 1
            event(self.path, {'event': 'reserve_before_post', 'id': identifier, 'attempt': attempt,
                  'overall_ordinal': self.attempts, 'retry': attempt > 1, 'reserved_nano_usd': held,
                  'cumulative_upper_nano_usd': self.total})
            return held

    def settle(self, identifier: str, attempt: int, held: int, actual: int, usage: dict[str, Any]) -> None:
        with self.lock:
            # User explicitly requested full reservation accounting for every retransmission.
            if attempt == 1:
                self.total += actual - held
            event(self.path, {'event': 'settle_usage' if attempt == 1 else 'retry_full_reservation_retained',
                  'id': identifier, 'attempt': attempt, 'usage': usage, 'usage_peak_nano_usd': actual,
                  'budget_charge_nano_usd': actual if attempt == 1 else held,
                  'cumulative_upper_nano_usd': self.total})
            if actual > held or self.total > self.plan['hard_nano_usd']:
                if attempt > 1:
                    self.total += max(0, actual - held)
                self.reason = 'reservation_or_hard_budget_violation'
                event(self.path, {'event': 'stop_new_posts', 'reason': self.reason,
                                  'cumulative_upper_nano_usd': self.total})
                self.stop.set()

    def record(self, value: dict[str, Any]) -> None:
        with self.lock:
            event(self.path, value)
