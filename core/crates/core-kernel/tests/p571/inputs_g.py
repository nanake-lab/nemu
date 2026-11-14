"""Frozen v2 §2.4 G inputs. No outcome, support or allocation is precomputed."""

from typing import Any


def g_case(row: dict[str, Any]) -> dict[str, Any]:
    s, j, branch = row["world"]["sign"], row["world"]["coordinate"], row["branch"]
    direction = [s if k == j else 0 for k in range(4)]
    candidate = dict(
        id="a",
        concept="ca",
        registry_version="P5-71-v2-registry",
        direction=direction,
        predicate={
            "zh": "synthetic situation",
            "en": "synthetic situation",
            "ja": "synthetic situation",
        },
        relation=None,
    )
    group = dict(
        id="old-g",
        first_position=1,
        source_keys=["src:old-g"],
        members=["old-e"],
        verifiable=True,
        appraisal=[v / 2 for v in direction],
        matches=[dict(candidate="a", mu=1)],
    )
    # §2.4 asks for genesis closeness then trust in proposal order, not the
    # canonical snapshot ordering of these two generated relationship IDs.
    seeds = ["rel:closeness:u-star", "rel:trust:u-star"]
    events = [
        dict(
            revision="g-setup",
            time_ns=10**9,
            candidates=[candidate],
            groups=[group],
            aliases=[],
            proposal_order=seeds + ["a"],
        )
    ]
    for i in range(1, 5):
        candidates, groups, aliases = [], [], []
        if branch in (0, 3):
            candidates = [dict(candidate, id="k", concept="ck")]
            if branch == 3:
                groups = [
                    dict(
                        group,
                        matches=[dict(candidate="a", mu=1), dict(candidate="k", mu=1)],
                    )
                ]
        elif branch == 1:
            candidates = [candidate]
        else:
            candidates = [dict(candidate, id="alias-a", concept="ca-alias")]
            aliases = [dict(alias="ca-alias", canonical="ca")]
        events.append(
            dict(
                revision=f"g-{i}",
                time_ns=(i + 1) * 10**9,
                candidates=candidates,
                groups=groups,
                aliases=aliases,
                proposal_order=seeds + ["a"] + (["k"] if branch in (0, 3) else []),
            )
        )
    return dict(
        id=row["id"],
        version=row["version"],
        resource="G",
        sign=s,
        coordinate=j,
        branch=branch,
        events=events,
        f_route_bindings=["old", "A1", "A2", "B"],
    )
