"""Hand-derived local examples for frozen §§2.3, 3.1, 5; never matrix results.

Expected fractions below are literals derived in REFERENCE-SELF-AUDIT.md.
The elementary-function stub specifies a dyadic half-step, not libm accuracy.
"""

from copy import deepcopy
from typing import Any

from arithmetic import Q
from f_oracle import F
from f_research import FResearch
from inputs_cf import cf_case
from inputs_f import f_case
from prepare import resources


def half(op: str, x: Q) -> Q:
    assert op == "expm1"
    return Q(0) if x == 0 else Q(-1, 2)


def event(
    i: int,
    op: str = "new",
    root: str = "r1",
    value: Q = Q(1, 4),
    at: int = 0,
    **extra: Any,
) -> dict[str, Any]:
    return dict(
        id=f"hand-{i}",
        ordinal=i,
        op=op,
        root=root,
        kind="record",
        target=[value, 0, 0, 0],
        time_ns=at,
        **extra,
    )


def model(version: str) -> FResearch:
    return FResearch(
        resources()[1],
        dict(
            id="hand",
            version=version,
            coordinate=0,
            theta=dict(b=[0] * 4, g=[1] * 4, lambda_bits=["3fe62e42fefa39ef"] * 4),
            b_admissions={},
        ),
        half,
    )


def b_admission() -> None:
    b = model("B")

    def apply(e: dict[str, Any], part: str | None) -> None:
        b.case["b_admissions"][str(e["ordinal"])] = dict(
            real_part=part, reason="hand truth"
        )
        b.step(e)

    apply(event(1), "initial")
    assert b.x[0] == Q(1, 4) and b.consumed == {"r1": ["initial"]}
    # Different numeric targets / theta / R / interpretation are not evidence.
    for i, label in enumerate(("theta", "R", "interpreter", "paraphrase"), 2):
        before = deepcopy(b.state())
        apply(dict(event(i, "update", value=Q(3, 4), at=10**9), revision=label), None)
        assert b.state() == before
    # Equal target, but a distinct lawful condition then counterevidence.
    for i, part, expected in (
        (6, "condition", Q(1, 2)),
        (7, "counterevidence", Q(3, 4)),
    ):
        apply(event(i, "update"), part)
        assert b.x[0] == expected
    before = deepcopy(b.state())
    apply(event(8, "update", value=Q(-1), at=10**9), "counterevidence")
    assert b.state() == before  # changed request/value cannot re-consume it
    for i, op in ((9, "revisit"), (10, "noop")):
        apply(event(i, op, at=10**9), None)
        assert b.state() == before
    for i, op in ((11, "new"), (12, "update")):
        apply(dict(event(i, op, root="internal"), kind="internal"), "invented")
        assert b.state() == before
    apply(event(13, "new", root="result"), "initial")
    assert b.x[0] == 1  # no global kappa discount on a new lawful result
    apply(event(14, "update", root="result", value=Q(0)), "zero-revision")
    assert b.consumed["result"] == ["initial", "zero-revision"]
    saved = deepcopy(b.consumed)
    for i, op in enumerate(("roundtrip", "remove_zero", "migrate", "merge"), 15):
        apply(event(i, op), None)
        assert b.consumed == saved
    apply(event(19, "revisit", root="result", at=10**9), None)
    assert b.anchor == 0 and b.projection(10**9)[0][0] == Q(1, 2)


def clipping() -> None:
    for sign in (-1, 1):
        for version in ("A1", "A2"):
            f = model(version)
            f.step(event(1, value=Q(sign * 3, 4)))
            f.step(event(2, root="r2", value=Q(sign * 3, 4)))
            assert f.x[0] == sign and f.h["r1"][0] == Q(sign * 3, 4)
            assert f.h["r2"][0] == Q(sign * (1 if version == "A1" else 3), 4)
            before = deepcopy(f.state())
            f.step(event(3, "revisit", "r2", Q(sign * 3, 4)))
            assert f.state() == before  # same time, root, recorded target
            f.step(event(4, "revisit", "r2", Q(sign * 3, 4), 10**9))
            assert f.x[0] == (sign if version == "A1" else Q(sign * 7, 8))
            assert f.h["r1"][0] == Q(sign * 3, 8)
    f = model("A1")
    for i, v in enumerate((1, -1, 1), 1):
        f.step(event(i, root=f"r{i}", value=Q(v)))
    f.step(event(4, "update", "r2", Q(0)))
    assert f.x[0] == 1 and [f.h[k][0] for k in ("r1", "r2", "r3")] == [1, -1, 1]
    # A0's sum-of-hidden-components readout is explicitly a failed comparison.
    a0, a1 = model("A0"), model("A1")
    for f in (a0, a1):
        f.step(event(1, value=Q(1)))
        f.step(event(2, root="r2", value=Q(1)))
    assert a0.projection(10**9)[0][0] == 1
    assert a1.projection(10**9)[0][0] == Q(1, 2)


def historical_and_roles() -> None:
    for version in ("old", "A1", "A2", "B"):
        f = model(version)
        f.case["b_admissions"] = {
            "1": dict(real_part="initial"),
            "2": dict(real_part=None),
            "3": dict(real_part="condition"),
        }
        e = dict(event(1), target_mode="current_theta", a=[Q(1, 4), 0, 0, 0])
        f.step(e)
        f.theta["g"] = [Q(2)] * 4
        f.step(dict(e, id="later", ordinal=2, op="revisit"))
        assert f.history["r1"]["recorded_target"][0] == Q(1, 4)
        assert f.x[0] == (Q(3, 8) if version == "old" else Q(1, 4))
        f.step(dict(e, ordinal=3, op="update"))
        assert f.history["r1"]["recorded_target"][0] == Q(1, 2)
    f = model("A1")
    f.step(dict(event(1), kind="internal"))
    f.step(dict(event(2, "update", value=Q(1, 2)), kind="internal"))
    assert f.x[0] == Q(1, 2)
    before = deepcopy(f.state())
    f.step(dict(event(3, "revisit"), kind="internal"))
    assert f.state() == before and f.last["fault"] == "internal revisit"
    f.step(event(4, "invalidate"))
    before = deepcopy(f.state())
    f.step(event(5, "revisit"))
    assert f.state() == before
    # Eviction/migration remove slots, not H's numeric target or visible x.
    f = model("A1")
    f.step(event(1, value=Q(0)))
    f.step(event(2, "remove_zero"))
    assert f.h == {} and f.history["r1"]["recorded_target"][0] == 0
    f.step(event(3, "revisit", value=Q(1)))
    assert f.x[0] == 0
    f.step(event(4, "update"))
    f.step(event(5, "migrate"))
    assert f.x[0] == Q(1, 4) and f.h == {} and f.anchor == 0
    f.step(event(6, "revisit", value=Q(1)))
    assert f.x[0] == Q(1, 2)  # known empty-slot migration cost, no guessed h
    old = model("old")
    for i in range(1, 4):
        old.step(event(i, "new" if i == 1 else "revisit", value=Q(1, 2)))
    assert old.x[0] == Q(7, 8) and old.internal_count == 2
    old.step(event(4, root="result", value=Q(-1, 2), source="action_result"))
    assert old.x[0] == Q(5, 8)  # n_issued=1, independent of internal n
    # Probe-only helper: first three are distinct roots; not the full driver.
    probe = F(dict(resources()[1], **{"lambda": Q(1)}), "A1", half)
    probe.step(dict(time_ns=0, target=Q(3, 4), root="r1"), 0)
    probe.step(dict(time_ns=0, target=Q(3, 4), root="r2"), 1)
    assert probe.x[0] == 1 and probe.h["r2"][0] == Q(1, 4)


def materialized_truth() -> None:
    def case(id_: str, variant: int, n: int) -> dict[str, Any]:
        return f_case(
            dict(
                id=id_,
                version="B",
                N=n,
                branch=0,
                world=dict(sign=1, coordinate=0, variant=variant),
            )
        )

    c = case("09-19", 0, 7)
    assert [a["real_part"] for a in c["b_admissions"].values()] == ["initial"] + [
        None
    ] * 6
    # Independent handwritten (g*a + s*mu*r*d, s*mu) at every referenced H row.
    expected = {
        1: (Q(3, 32), Q(1, 16)),
        2: (Q(3, 16), Q(1, 8)),
        3: (Q(9, 32), Q(3, 16)),
        5: (Q(-3, 32), Q(3, 16)),
        6: (Q(-1, 8), Q(1, 4)),
        7: (Q(-1, 8), Q(1, 4)),
    }
    for version in ("old", "A1", "A2", "B"):
        f = FResearch(resources()[1], dict(c, version=version), half)
        for e in c["events"]:
            if e["ordinal"] not in expected:
                continue
            v = f.target(e)
            assert (v[0], v[4]) == expected[e["ordinal"]]
            assert f.last["resolved_inputs"]["theta_revision"] == e["theta_revision"]
            assert (
                f.last["resolved_inputs"]["effects_revision"] == e["effects_revision"]
            )
    for version, answer in (
        ("wrong-theta", Q(1, 32)),
        ("wrong-R", Q(-5, 32)),
        ("wrong-interpretation", Q(9, 32)),
    ):
        f = FResearch(resources()[1], dict(c, version=version), half)
        assert f.target(c["events"][4])[0] == answer
    for variant in (5, 7):  # same target sign, distinct condition / counterevidence
        c = case("09-07", variant, 4)
        assert c["events"][0]["target"] == c["events"][1]["target"]
        assert c["b_admissions"]["2"]["real_part"] == "real-revision-2"
    c = case("09-07", 3, 4)
    assert c["b_admissions"]["2"]["real_part"] is None  # invalid certificate
    c = case("09-09", 5, 3)
    assert c["b_admissions"]["2"]["real_part"] == c["b_admissions"]["3"]["real_part"]
    c = cf_case(
        dict(id="C-02", version="A-B", N=16, branch=0, world=dict(sign=1, coordinate=0))
    )
    assert {c["b_admissions"][str(i)]["real_part"] for i in range(7, 17)} == {
        "real-revision-7"
    }


def maintenance_and_controls() -> None:
    f = model("A1")
    f.step(event(1))
    f.step(event(2, root="r2", value=Q(1, 2)))
    f.step(event(3, "merge", members=["r2", "r1"]))
    assert f.x[0] == Q(3, 4) and f.h == {"r1": [Q(3, 4), 0, 0, 0]}
    assert f.targets["r1"][0] == Q(1, 4)  # earliest position, not member-list order
    f.step(event(4, "revisit"))
    assert f.x[0] == Q(1, 4)
    for control in ("F-off", "x-zero"):
        f = model("A1")
        f.case["negative_control"] = control
        f.step(event(1))
        assert f.x[0] == 0  # deliberately fails positive first-action value 1/4


def run() -> None:
    for check in (
        b_admission,
        clipping,
        historical_and_roles,
        materialized_truth,
        maintenance_and_controls,
    ):
        check()
        print(f"hand F: {check.__name__} passed")
