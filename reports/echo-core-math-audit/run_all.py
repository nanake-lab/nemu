"""Run every Echo Core math-audit check and print a result table.

Usage (repository root):  python reports/echo-core-math-audit/run_all.py
Exit code 0 when every property holds and every counterexample is reproduced.
Standard library only; deterministic seeds.
"""

from __future__ import annotations

import platform
import sys
import time

import check_bounds
import check_budget
import check_ids
import check_metric
import check_partition
import check_phi
import check_schedule
import check_stats
import check_structure
from common import Result

MODULES = [
    check_phi,
    check_bounds,
    check_stats,
    check_budget,
    check_metric,
    check_partition,
    check_schedule,
    check_structure,
    check_ids,
]


def main() -> int:
    print(f"python {platform.python_version()} on {platform.system()} {platform.machine()}")
    results: list[Result] = []
    start = time.perf_counter()
    for module in MODULES:
        for check in module.CHECKS:
            results.append(check())
    elapsed = time.perf_counter() - start
    for r in results:
        status = "OK  " if r.ok else "FAIL"
        print(f"[{status}] {r.check_id:<7} {r.kind:<14} {r.spec_ref}")
        print(f"         {r.detail}")
    failed = [r.check_id for r in results if not r.ok]
    props = sum(r.kind == "property" for r in results)
    print(f"\n{len(results)} checks ({props} properties, {len(results) - props} counterexamples) "
          f"in {elapsed:.1f} s; failed: {failed or 'none'}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
