"""Software guards, not S1 experimental samples."""

from copy import deepcopy
from typing import Any

from arithmetic import Q
from criteria import distance_failure
from hand_f import event, model
from s1_analysis import Analysis
from s1_refs import references, check, wrong_reference_checks, objects


def run() -> None:
    from s1_run import stored_fast

    view = dict(x=[], h={}, targets={}, anchor=0)
    assert (
        stored_fast(dict(state=view), dict(layer="independent_research", family="F"))
        == view
    )
    assert distance_failure("A1", dict(extended_nonexpansive=False)) is not None
    assert distance_failure("A1", dict(extended_nonexpansive=True)) is None
    assert distance_failure("old", dict(extended_nonexpansive=False)) is None
    for version in ("A1", "A2"):
        m = model(version)
        analysis = Analysis(m)
        for e in (
            event(1, value=Q(3, 4)),
            event(2, root="r2", value=Q(3, 4)),
            event(3, "revisit", root="r2", at=10**9),
            event(4, "update", root="r2", value=Q(0), at=10**9),
        ):
            before = deepcopy(m)
            m.step(e)
            result = analysis.observe(before, m, e, True)
            assert result["history_contains_h"]
            assert result["residual_enclosed"]
        h: dict[str, Any] = dict(source_position=1, objects=objects(m.theta, event(1)))
        audit = dict(record=references(h["objects"]), synthetic_h=h)
        assert check(h, audit) is None
        assert len(wrong_reference_checks(h, audit)) == 3
    # Hand construction of the fixed separating input, no libm or DUT recurrence.
    assert Q(1, 6) < Q(43, 256) < Q(11, 64)
    print(
        "S1 software: distance gate, analytic guards, three isolated wrong refs passed"
    )


if __name__ == "__main__":
    run()
