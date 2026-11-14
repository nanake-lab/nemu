"""Self-audit examples and frozen row-order check; no formal driver invocation."""

from collections import Counter
import json

from hand_f import run as f_checks
from hand_cg import run as cg_checks
from materialize import deferred, ordered_plan
from prepare import ledger


def run() -> None:
    from s1_checks import run as s1_checks

    s1_checks()
    f_checks()
    cg_checks()
    source, ordered = ledger() + deferred(), ordered_plan()
    assert Counter(json.dumps(r, sort_keys=True) for r in source) == Counter(
        json.dumps(r, sort_keys=True) for r in ordered
    )
    main_ids = list(
        dict.fromkeys(
            r["id"]
            for r in ordered
            if r["id"]
            in {
                *(f"07-{i:02}" for i in range(1, 13)),
                *(f"09-{i:02}" for i in range(1, 20)),
                *(f"C-{i:02}" for i in range(1, 5)),
            }
        )
    )
    assert main_ids == (
        [f"07-{i:02}" for i in range(1, 13)]
        + [f"09-{i:02}" for i in range(1, 20)]
        + [f"C-{i:02}" for i in range(1, 5)]
    )
    assert len(ordered) == 49046
    print("hand materialization: row order and unchanged ledger multiset passed")


if __name__ == "__main__":
    run()
