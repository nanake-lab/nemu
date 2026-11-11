"""Independent frozen §2.4 G research model.

The domain is exactly the registered one-group candidate/backfill experiment;
this is not a substitute implementation of arbitrary production G. Formulae:
v2 §5.2 support/shares and ADR0019 first-allocation q and stored-q revision.
No production or previous-oracle imports and no DUT-generated genesis.
"""

from copy import deepcopy
from typing import Any

from arithmetic import Q, rounded as r
from c_oracle import vec
from f_research import scalar_bits


def order(value: str) -> tuple[int, bytes]:
    raw = value.encode("utf-8")
    return len(raw), raw


class GResearch:
    def __init__(
        self, p: dict[str, Any], case: dict[str, Any], binding: dict[str, Any]
    ) -> None:
        self.p, self.case, self.binding = p, case, binding
        self.t: dict[str, dict[str, Any]] = {}
        self.groups: dict[str, dict[str, Any]] = {}
        self.revision = "r0"
        self.last: dict[str, Any] = {}
        texts = [
            (
                "这件事涉及此人：此人说话、被说到或被提到",
                "The event involves this person as speaker, addressee or someone mentioned",
                "この出来事にこの人が話し手、聞き手、または話題の人として関わっている",
            ),
            (
                "她依靠此人说的话或做的事",
                "She relies on what this person says or does",
                "彼女がこの人の言葉や行動に頼っている",
            ),
        ]
        for index, kind in enumerate(("closeness", "trust")):
            id_ = f"rel:{kind}:{p['caretaker']['person_id']}"
            direction = (
                [Q(1), Q(0), Q(0), Q(0)]
                if index == 0
                else [
                    Q(0),
                    Q(0),
                    scalar_bits("3fe6a09e667f3bcd"),
                    scalar_bits("3fe6a09e667f3bcd"),
                ]
            )
            t = self.initial(
                id_,
                p["relation_concepts"][index],
                direction,
                texts[index],
                Q(p["caretaker"]["attachment_prior"]) if index == 0 else Q(0),
                Q(p["caretaker"]["attachment_prior"]) if index == 1 else Q(0),
            )
            t["relation"] = dict(
                schema_version=1,
                person_id=p["caretaker"]["person_id"],
                kind=1 if index == 0 else 2,
            )
            if index == 0:
                t["attachment_prior"] = Q(p["caretaker"]["attachment_prior"])
            self.t[id_] = t

    def initial(
        self,
        id_: str,
        concept: str,
        direction: list[Q],
        texts: tuple[str, ...],
        prior: Q = Q(0),
        evidence: Q = Q(0),
    ) -> dict[str, Any]:
        c = r(Q(self.p["n0"]) + Q(self.p["kappa1"]))
        return dict(
            schema_version=1,
            tendency_id=id_,
            concept=dict(
                schema_version=1,
                concept_id=concept,
                registry_version=self.p["registry_version"],
            ),
            predicate=[dict(schema_version=1, zh=texts[0], en=texts[1], ja=texts[2])],
            direction=vec(direction),
            evidence_a=evidence,
            evidence_b=r(c + evidence),
            evidence_absolute=evidence,
            h_hat=r(r(Q(self.p["h0"]) * Q(self.p["n_h"])) / Q(self.p["n_h"])),
            group_count="0",
            flip_count="0",
            pi=r(r(prior + evidence) / r(r(prior + c) + evidence)),
            typical_magnitude=Q(0),
            magnitude_sum=Q(0),
            magnitude_weight=Q(0),
        )

    def one_group(self, t: dict[str, Any], share: Q) -> None:
        # Every registered ordinary tendency starts with zero evidence. Its
        # sole support is +1, intensity=0.5/0.25=2 and projection=+0.5;
        # s mirror changes a and d together, so these scalars are unchanged.
        intensity = r(Q(1, 2) / Q(self.p["theta_sigma"][self.case["coordinate"]]))
        w = r(r(r(intensity * 1) * 1) * share)
        a = r(r(Q(0) * r(1 - Q(self.p["h0"]))) + r(w * 1))
        v = r(r(Q(0) * r(1 - Q(self.p["h0"]))) + r(w * 1))
        c = r(Q(self.p["n0"]) + Q(self.p["kappa1"]))
        # Magnitude count weights are q*old + mu*p, independently of W.
        mass = r(Q(0) + 1)
        summed = r(Q(0) + Q(1, 2))
        t.update(
            evidence_a=a,
            evidence_absolute=v,
            evidence_b=r(c + v),
            pi=r(a / r(c + v)),
            group_count="1",
            flip_count="0",
            last_sign=2,
            h_hat=r(r(Q(self.p["h0"]) * Q(self.p["n_h"])) / r(Q(self.p["n_h"]) + 1)),
            magnitude_sum=summed,
            magnitude_weight=mass,
            typical_magnitude=r(summed / mass),
        )

    def step(self, e: dict[str, Any], index: int) -> None:
        if self.case.get("negative_control") == "G-frozen" and index > 0:
            self.last = dict(
                changes=[],
                counterfactual={},
                after_additions={},
                frozen_negative_control=True,
            )
            return
        before = deepcopy(self.t)
        groups_before = deepcopy(self.groups)
        aliases = {a["alias"]: a["canonical"] for a in e["aliases"]}
        for candidate in e["candidates"]:
            concept = aliases.get(candidate["concept"], candidate["concept"])
            if any(t["concept"]["concept_id"] == concept for t in self.t.values()):
                continue
            pred = candidate["predicate"]
            self.t[candidate["id"]] = self.initial(
                candidate["id"],
                concept,
                [Q(v) for v in candidate["direction"]],
                (pred["zh"], pred["en"], pred["ja"]),
            )
        processed: set[str] = set()
        pending: list[str] = []
        for group in e["groups"]:
            ids = sorted([m["candidate"] for m in group["matches"]], key=order)
            previous = self.groups.get(group["id"])
            previous_ids = (
                []
                if previous is None
                else [a["tendency_id"] for a in previous["allocations"]]
            )
            if ids == previous_ids:
                continue
            # Exact quantization of independent rounded (1+pi)/2 weights onto
            # the 2^-52 share lattice, largest remainder then canonical ID.
            weights = [r(r(1 + self.t[id_]["pi"]) / 2) for id_ in ids]
            total = sum(weights, Q(0))
            quotas = [w * 2**52 / total for w in weights]
            counts = [q.numerator // q.denominator for q in quotas]
            ranked = sorted(
                range(len(ids)), key=lambda k: (-(quotas[k] - counts[k]), order(ids[k]))
            )
            for k in ranked[: 2**52 - sum(counts)]:
                counts[k] += 1
            shares = [Q(v, 2**52) for v in counts]
            allocations = []
            for id_, share in zip(ids, shares):
                allocations.append(
                    dict(
                        schema_version=1,
                        tendency_id=id_,
                        support=Q(1),
                        intensity=Q(2),
                        share=share,
                        ordinal="0",
                        mu=Q(1),
                        projection=Q(1, 2),
                        actual_retention=r(1 - Q(self.p["h0"])),
                    )
                )
                if id_ in previous_ids:
                    self.one_group(self.t[id_], share)
                else:
                    pending.append(id_)
                processed.add(id_)
            self.groups[group["id"]] = dict(
                schema_version=1,
                group_id=group["id"],
                members=group["members"],
                source_key=group["source_keys"][0],
                credibility=Q(1),
                verifiability=Q(1),
                allocations=allocations,
            )
        counterfactual = {id_: t["pi"] for id_, t in self.t.items()}
        for id_ in pending:
            allocation = next(
                a
                for g in self.groups.values()
                for a in g["allocations"]
                if a["tendency_id"] == id_
            )
            self.one_group(self.t[id_], allocation["share"])
        after = {id_: t["pi"] for id_, t in self.t.items()}
        changes = []
        for id_ in sorted(self.t, key=order):
            if before.get(id_) == self.t[id_]:
                continue
            change = dict(
                schema_version=1,
                tendency_id=id_,
                pi_after=self.t[id_]["pi"],
                evidence_groups=["old-g"] if id_ in processed else [],
            )
            if id_ in before:
                change["pi_before"] = before[id_]["pi"]
            changes.append(change)
        if changes or groups_before != self.groups:
            self.revision = e["revision"]
        self.last = dict(
            changes=changes,
            counterfactual=counterfactual,
            after_additions=after,
            capacity_audits=[],
            capacity_notice=None,
        )

    def state(self) -> dict[str, Any]:
        return dict(
            r=dict(
                schema_version=1,
                revision=self.revision,
                law_version=self.binding,
                tendencies=[self.t[k] for k in sorted(self.t, key=order)],
            ),
            groups=[self.groups[k] for k in sorted(self.groups, key=order)],
        )
