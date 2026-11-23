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
    merge_005136()
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


def merge_005136() -> None:
    """S1 §4: frozen 005136's merge residual belongs to this step too."""
    from inputs_f import f_case

    case = f_case(
        dict(id="09-11", version="A1", N=4, world=dict(sign=1, coordinate=0, variant=6))
    )
    m = model("A1")
    analysis = Analysis(m)
    results = []
    # The fixed half-step primitive reproduces this fixture's locked pull=1/2;
    # it is not a test of libm or a rerun of the DUT.
    for e in case["events"][:3]:
        before = deepcopy(m)
        m.step(e)
        results.append(analysis.observe(before, m, e, e["op"] != "merge"))
    expected = Q(1, 2**56)  # exact sum of stored slots minus fl(sum)
    assert results[1]["decomposition_residual"][0] == str(-expected)
    assert results[2]["decomposition_residual"][0] == "0"
    assert results[2]["local_error"] == [[str(expected)] * 2] + [["0", "0"]] * 3
    assert results[2]["cumulative_error"][0] == ["0", str(Q(1, 2**54))]


if __name__ == "__main__":
    run()
