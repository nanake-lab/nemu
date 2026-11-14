"""Independent hand examples: qualification, lifetime mass, G allocation, CF.

Python math supplies elementary functions only; asserted answers are literal
dyadic values / hand binary64 encodings. This is no formal matrix execution.
"""

import math
from copy import deepcopy
from typing import Any

from arithmetic import Q, bits
from c_oracle import clamp
from c_research import CResearch
from cf_research import CFResearch
from g_research import GResearch
from hand_f import event as fevent, half
from inputs_g import g_case
from prepare import resources
from sensitivity import BaselinePath, arithmetic, candidate


def elementary(op: str, x: Q) -> Q:
    return Q(
        {"exp": math.exp, "expm1": math.expm1, "log": math.log, "sqrt": math.sqrt}[op](
            float(x)
        )
    )


def event(i: int, a: float | str = 0.5, **extra: Any) -> dict[str, Any]:
    return dict(
        id=f"hand-c-{i}",
        ordinal=i,
        time_ns=i * 10**9,
        running_ns=i * 10**9,
        a=a,
        eligible=True,
        source="other_person",
        **extra,
    )


def baselines() -> None:
    # P0=1,b0=0,rho=1/2,a=(1/2,-1/2). Values were derived by hand.
    expected = {
        "A": [
            (Q(1, 2), Q(1), Q(1, 2), "3fd5555555555555"),
            (Q(1, 4), Q(3, 2), Q(-1, 4), "bfc2492492492492"),
        ],
        "old": [
            (Q(1), Q(1), Q(1, 2), "3fd0000000000000"),
            (Q(1), Q(3, 2), Q(-1, 4), "bfb999999999999a"),
        ],
        "B": [
            (Q(1), Q(1), Q(1, 2), "3fd0000000000000"),
            (Q(1), Q(2), Q(0), "0000000000000000"),
        ],
    }
    for version, values in expected.items():
        case = dict(id="hand", version=version, p0=1, b0=0, rho=0.5)
        c, path = CResearch(resources()[0], case, elementary), BaselinePath(case)
        for i, (p, s, u, b) in enumerate(values, 1):
            ev = event(i, 0.5 if i == 1 else -0.5)
            before = c.c.values["baseline"]["connection"]
            c.step(ev)
            path.step(ev)
            assert c.last["fault"] is None
            m = c.metrics()
            assert (m["P"], m["S"], m["U"][0]) == (p, s, u)
            assert bits(c.c.values["baseline"]["connection"]) == b
            assert (path.p, path.s, path.u, bits(path.b)) == (p, s, u, b)
            assert (
                abs(c.c.values["baseline"]["connection"] - before)
                <= c.last["bounds"]["limits"][0]
            )
            assert m["stored_in_enclosure"] and m["real_in_enclosure"]
        if version == "B":
            assert c.c.mass == 1 and c.c.e["weight"] == Q(3, 2)
            assert c.c.e["reaction_sum"][0] == Q(-1, 4)  # other estimators retain rho


def c_gates() -> None:
    p = resources()[0]
    for version in ("old", "A", "B"):
        c = CResearch(
            p, dict(id="hand", version=version, p0=1, b0=0, rho=0.5), elementary
        )
        for i, source in enumerate(
            (
                "excluded",
                "self_action",
                "program_result",
                "self_narrative",
                "desktop",
                "missing_A",
                "duplicate_contact",
            ),
            1,
        ):
            before = deepcopy(c.state())
            c.step(dict(event(i), source=source, eligible=False))
            assert c.state() == before and c.qualified == []
        c.step(event(8, -0.5))
        assert c.qualified == [8] and c.metrics()["S"] == 1
        before = deepcopy(c.state())
        for i, a in enumerate(("nan", "inf", "-inf"), 9):
            c.step(event(i, a))
            assert c.last["fault"] == "NonFinite" and c.state() == before
        c.step(event(12, 0.5))  # independent positive evidence also admitted
        assert c.qualified == [8, 12]
    # Governance correction rebuilds history, a recantation merely adds evidence.
    for invalidate, expected in ((False, Q(-1, 7)), (True, Q(-1, 3))):
        c = CResearch(p, dict(id="hand", version="A", p0=1, b0=0, rho=0.5), elementary)
        c.step(event(1))
        second = event(2, -0.5)
        if invalidate:
            second["invalidate_prior_ordinals"] = [1]
        c.step(second)
        assert c.c.values["baseline"]["connection"] == Q(float(expected))
        assert c.qualified == ([2] if invalidate else [1, 2])
    # Open runs add nothing, zero lengths only count, positive running duration adds one.
    c = CResearch(p, dict(id="hand", version="A", p0=1, b0=0, rho=0.5), elementary)
    c.step(event(1))
    assert c.c.e["duration_mass"] == [0] * 4
    c.step(dict(event(2, -0.5), running_ns=10**9))
    assert (
        c.c.e["zero_duration_count"] == ["1"] * 4 and c.c.e["duration_mass"] == [0] * 4
    )
    c.step(dict(event(3, 0.5), time_ns=100 * 10**9, running_ns=2 * 10**9))
    assert c.c.e["duration_mass"] == [1] * 4 and c.c.e["duration_log_sum"] == [0] * 4
    # The sign uses the previous baseline: a remains positive but a-b is negative.
    c = CResearch(p, dict(id="hand", version="A", p0=1, b0=0, rho=0.5), elementary)
    c.step(event(1, 1.0))
    c.step(event(2, 0.5))
    assert c.c.e["run_sign"] == ["-1"] * 4 and c.c.e["duration_mass"] == [1] * 4
    for lo, hi in (
        (Q(p["gain_min"]), Q(p["gain_max"])),
        (Q(p["recovery_min"]), Q(p["recovery_max"])),
    ):
        assert [clamp(x, lo, hi) for x in (lo - 1, lo, hi, hi + 1)] == [lo, lo, hi, hi]
    alarm = dict(p, jump_notice_bits="0000000000000001")
    c = CResearch(alarm, dict(id="hand", version="A", p0=1, b0=0, rho=0.5), elementary)
    c.step(event(1))
    assert c.last["signals"] == ["Jump"] and c.metrics()["S"] == 1


def boundaries() -> None:
    for source, expected in (
        ("0000000000000001", "0000000000000000"),
        ("0000000000000003", "0000000000000002"),
    ):
        case = dict(
            version="A",
            p0=1,
            b0=0,
            rho=0.5,
            initial=dict(P_bits=source, S=1, U=0, rho=0.5),
        )
        assert arithmetic(case)["rounded_bits"] == expected
        path = BaselinePath(case)
        path.step(event(1))
        assert bits(path.p) == expected and path.fault is None
    for point in ("equal", "next_up", "nan", "inf", "-inf"):
        c = dict(
            first_normal_step=dict(p0=1, b0=0, rho=0.5, a=0.5, time_ns=10**9),
            coordinate=0,
            point=point,
            version="model",
        )
        result = candidate(resources()[0], c, elementary)
        assert result["status"] == "passed" and result["actual_accept"] == (
            point == "equal"
        )
    for mutant, point in (
        ("greater-equal", "equal"),
        ("clip", "next_up"),
        ("ignore-nonfinite", "nan"),
    ):
        assert (
            candidate(resources()[0], dict(c, point=point, version=mutant), elementary)[
                "status"
            ]
            == "failed"
        )


def g_shares() -> None:
    for branch in range(4):
        case = g_case(
            dict(
                id="hand", version="G", world=dict(sign=1, coordinate=0), branch=branch
            )
        )
        g = GResearch(resources()[2], case, {})
        caretaker = deepcopy(g.t)
        assert [t["pi"] for t in caretaker.values()] == [Q(15, 16)] * 2
        g.step(case["events"][0], 0)
        assert g.t["a"]["evidence_a"] == 2 and g.t["a"]["evidence_b"] == 6
        assert bits(g.t["a"]["pi"]) == "3fd5555555555555"
        assert g.t["a"]["typical_magnitude"] == Q(1, 2)
        before = deepcopy(g.state())
        g.step(case["events"][1], 1)
        if branch == 0:
            assert g.t["k"]["pi"] == 0 and g.t["k"]["group_count"] == "0"
            assert g.groups == {x["group_id"]: x for x in before["groups"]}
        elif branch in (1, 2):
            assert g.state() == before
        else:
            allocations = g.groups["old-g"]["allocations"]
            assert [a["share"] for a in allocations] == [
                Q(2573485501354569, 2**52),
                Q(1930114126015927, 2**52),
            ]
            assert sum(a["share"] for a in allocations) == 1
            assert [a["ordinal"] for a in allocations] == ["0", "0"]
            assert all(
                bits(a["actual_retention"]) == "3feccccccccccccd" for a in allocations
            )
            assert g.t["k"]["pi"] > 0 and g.t["k"]["group_count"] == "1"
        after = deepcopy(g.state())
        g.step(case["events"][2], 2)
        assert g.state() == after and all(g.t[k] == v for k, v in caretaker.items())
        frozen = GResearch(resources()[2], dict(case, negative_control="G-frozen"), {})
        frozen.step(case["events"][0], 0)
        frozen.step(case["events"][1], 1)
        if branch == 3:
            assert "k" not in frozen.t and frozen.state() != g.state()


def interactions() -> None:
    cp, fp, _ = resources()
    for version in ("oldC-oldF", "A-oldF", "oldC-A1", "A-A1", "B-A1", "A-A2", "A-B"):
        case = dict(
            id="hand",
            version=version,
            p0=1,
            b0=0,
            rho=0.5,
            coordinate=0,
            b_admissions={"1": dict(real_part="initial"), "3": dict(real_part=None)},
        )
        cf = CFResearch(cp, fp, case, elementary)
        cf.f.fn = half
        cf.step(dict(time_ns=0, c=None, f=fevent(1, value=Q(1, 2))))
        saved = deepcopy(cf.f.targets)
        cf.step(dict(time_ns=10**9, c=event(2), f=None))
        assert cf.f.x[0] == Q(1, 4) and cf.f.targets == saved
        if cf.f.base in ("A1", "A2"):
            assert cf.f.h["r1"][0] == Q(1, 4)
        assert cf.c.qualified == [2]
        c_before = deepcopy(cf.c.state())
        cf.step(dict(time_ns=10**9, c=None, f=fevent(3, "revisit", at=10**9)))
        assert cf.c.state() == c_before and cf.f.targets == saved
        # S1 old reference: first internal n=0, so 1/4 + 1/2 = 3/4.
        assert cf.f.x[0] == (
            Q(1, 4) if cf.f.base == "B" else Q(3, 4) if cf.f.base == "old" else Q(1, 2)
        )


def run() -> None:
    for check in (baselines, c_gates, boundaries, g_shares, interactions):
        check()
        print(f"hand C/G/CF: {check.__name__} passed")
