"""Prospective policy boundaries; old runs never enter this assessment."""
from pathlib import Path
import sys
import pytest
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from pressure import assess  # noqa: E402


def test_exact_rss_limit_and_unchanged_allocator() -> None:
    mib = 2**20
    samples = [(0., 99*mib, 7), (1200., 100*mib, 7), (1500., 101*mib, 7), (1800., 102*mib, 7)]
    result = assess(samples)
    assert result["rss_20_30_min_slope_mib_hour"] == 12
    assert result["passed_30_min"] and not result["extension_required"]
    assert assess([*samples[:-1], (1800., 103*mib, 7)])["extension_required"]
    assert assess([*samples[:-1], (1800., 102*mib, 8)])["extension_required"]


def test_missing_window_cannot_pass() -> None:
    with pytest.raises(ValueError, match="TOO_FEW"):
        assess([(0., 1, 1), (1800., 1, 1)])
