"""Synthetic ADR 0048 contract fixtures. No registry or history is opened."""

from __future__ import annotations

from copy import deepcopy
from typing import Any

from fixture_inputs import LIMITS, sample

Row = dict[str, Any]


def msg(**fields: Any) -> Row:
    return {"schema_version": 1, **fields}


def evidence_sample(name: str, seed: int) -> Row:
    version = f"registry-{seed}"
    word = ["话", "声🙂", "e\u0301"][seed % 3]

    def obj(tag: str) -> Row:
        return sample("ObjectRef", seed) | {
            "object_type": tag,
            "resource_version": version,
        }

    def rec(key: str) -> Row:
        return msg(branch_id=version, record_id=key, content=obj("echo.user-text"))

    event = rec(word)
    field = msg(field_number=2)
    span = msg(start="0", end=str(len(word.encode())))
    loc = msg(source=event, path=[field], text=span)
    meta = msg(
        derivation=1,
        epistemic_type=1,
        resolution_state=2,
        representation_version=version,
        locations=[loc],
        roots=[event],
        transformations=[],
        producer=obj("producer"),
        validity=obj("grant"),
    )
    concept = msg(concept_id=word, registry_version=version)
    other = concept | {"concept_id": word + "-alias"}
    role = msg(role="speaker", node_kind=1, mandatory=True)
    revision = msg(
        concept_id=word,
        lifecycle=seed % 3 + 1,
        governance=obj("review"),
        grounding=[loc],
        label=sample("TrilingualText", seed),
        definition=sample("TrilingualText", seed),
        positive_examples=[],
        negative_examples=[],
        roles=[role],
        allows_reported_text=True,
        constraints=[],
    )
    link = msg(origin=other, target=concept, kind="canonical", basis=obj("review"))
    ref = msg(id="speaker", source=meta, referent=concept)
    claim = msg(
        id="claim",
        predicate=concept,
        roles=[msg(role="speaker", node_ref="speaker")],
        polarity=seed != 1,
        source=meta,
        reported_text=loc,
        mode=1,
        time_kind=seed % 3 + 1,
    )
    if claim["time_kind"] == 3:
        claim["time_fact"] = obj("time")
    act = msg(id=word + "-act", source=meta, interaction_kind="speech")
    reg = obj("echo.concept-registry")
    entry = msg(
        payload_tag="echo.user-text",
        payload_schema=1,
        path=[field],
        predicate=concept,
        role="speaker",
    )
    voice = entry | {"payload_tag": "echo.user-utterance"}
    outcomes: list[Row] = [
        {"accepted": obj("graph")},
        {"mu": 0.0},
        {"unknown_reason": "unobserved"},
    ]
    coverage = msg(target=word, **outcomes[seed % 3])
    binding = sample("ProposalBindingV2", seed)
    context = obj("context")
    binding.update(
        branch_id=version, c_core_digest=context["digest"], read_set=[context]
    )
    env = binding["environment"]
    stage = seed % 3 + 1
    resource = obj("interpreter")
    env[["irl", "nature_appraiser", "situation_matcher"][stage - 1]] = resource
    adoption = msg(
        event=event,
        interpretation=rec("interpretation"),
        stage=stage,
        outcome=1,
        binding=binding,
        producer_resource=resource,
        candidate=obj("candidate"),
        actual_output=obj("output"),
        context=context,
        definitions=[],
        isolation=obj("isolation"),
        governance=obj("review"),
        coverage=[],
        transaction_id=version,
        replaced_targets=[],
        registry_before=env["registry"],
        staged_revisions=[],
    )
    if stage == 2:
        adoption["appraisal"] = sample("NatureVector", seed)
    else:
        adoption["coverage"] = [coverage]
    source, kind, tag, access = [
        (4, 2, "echo.desktop-observation", 2),
        (5, 4, "echo.body-action-result", 3),
        (7, 8, "echo.system-maintenance-record", 1),
    ][seed % 3]
    rule = msg(
        source=source,
        event_kind=kind,
        payload_type=tag,
        exogenous=False,
        self_narrative=False,
        growth_access=access,
    )
    scoped = msg(scope=loc, value=seed % 3 + 1, evidence=[event])
    if scoped["value"] == 3:
        scoped["reason"] = "conflicting reports"
    req = msg(scope=loc, expected=1 + seed % 2)
    premise = msg(
        target=concept,
        occurrence_scopes=[req],
        truth_scopes=[],
        subjective_scopes=[],
        definition=obj("target"),
    )
    attrs = msg(
        event_id=word,
        rule_table_version=version,
        exogenous=False,
        self_narrative=False,
        source_key=word,
        event=event,
        rule_table=obj("echo.attribute-rule-table"),
        applied=rule,
        source=meta,
        governance=obj("review"),
        occurred=[scoped],
        truth=[scoped],
        subjective=[scoped | ({"experiencer": word} if scoped["value"] == 1 else {})],
        targets=[premise],
        inputs=[event],
    )
    if source == 5:
        attrs["cause"] = event
    rows = {
        "SourceField": msg(
            field_number=seed + 1, **({"element": str(seed)} if seed else {})
        ),
        "SourceLocation": loc,
        "SourceTransform": msg(
            inputs=[event], output=rec("derived"), rule=obj("transform")
        ),
        "SourceMetadataV2": meta,
        "ConceptRole": role | {"role": word},
        "ConceptRevision": revision,
        "ConceptLink": link,
        "ConceptRegistry": msg(
            version=version,
            entries=[revision, revision | {"concept_id": other["concept_id"]}],
            links=[],
            aliases=[link],
            relation_kinds=[],
            interaction_kinds=["speech"],
            edge_kinds=[],
        ),
        "RefV2": ref,
        "ClaimV2": claim,
        "ActV2": act,
        "SemanticGraphV2": msg(
            id=version,
            references=[ref],
            statements=[claim],
            actions=[act],
            edges=[],
            registry=reg,
        ),
        "TextAdapterEntry": [entry, voice, entry][seed % 3],
        "TextAdapterTable": msg(registry=reg, entries=[entry, voice]),
        "StageCoverage": coverage,
        "StageAdoption": adoption,
        "AttributeRule": rule,
        "AttributeRuleTable": msg(rules=[rule]),
        "ScopedVerification": scoped,
        "VerificationRequirement": req,
        "TargetPremise": premise,
        "EventAttributesRecordV2": attrs,
    }
    if name == "EventAttributesRecordV2":
        event["content"] = obj(tag)
    return rows[name]


def evidence_illegal() -> list[Row]:
    cases: list[tuple[str, str, Row]] = [
        ("SourceLocation", "empty-range", {"text": msg(start="2", end="2")}),
        ("SourceMetadataV2", "unbound-roots", {"roots": []}),
        (
            "ConceptRevision",
            "stable-without-grounding",
            {"lifecycle": 2, "grounding": []},
        ),
        ("ClaimV2", "missing-time-proof", {"time_kind": 3}),
        ("SemanticGraphV2", "dangling-role", {"references": []}),
        ("TextAdapterEntry", "unsupported-touch", {"payload_tag": "echo.touch-v2"}),
        ("TextAdapterTable", "missing-voice", {"entries": []}),
        ("StageCoverage", "unknown-and-known", {"mu": 0.0}),
        ("StageAdoption", "failure-retained-output", {"outcome": 3, "reason": "fault"}),
        ("AttributeRule", "desktop-external", {"exogenous": True}),
        ("AttributeRuleTable", "empty-table", {"rules": []}),
        ("ScopedVerification", "known-without-evidence", {"evidence": []}),
        ("VerificationRequirement", "unknown-premise", {"expected": 3}),
        ("TargetPremise", "no-premises", {"occurrence_scopes": []}),
        ("EventAttributesRecordV2", "missing-input", {"inputs": []}),
    ]
    registry = evidence_sample("ConceptRegistry", 0)
    reverse = deepcopy(registry["aliases"][0])
    reverse["origin"], reverse["target"] = reverse["target"], reverse["origin"]
    cases.append(
        ("ConceptRegistry", "alias-cycle", {"aliases": registry["aliases"] + [reverse]})
    )
    extra: list[Row] = []
    for q in evidence_legal():
        value = deepcopy(q["value"])
        if q["name"] == "SourceMetadataV2":
            value["transformations"][0]["inputs"] = []
            extra.append(
                dict(label="empty-transform-input", request=q | {"value": value})
            )
        if q["name"] == "EventAttributesRecordV2":
            for label, axis, key in [
                ("missing-experiencer", "subjective", "experiencer"),
                ("unknown-without-reason", "truth", "reason"),
            ]:
                broken = deepcopy(value)
                del broken[axis][0][key]
                extra.append(dict(label=label, request=q | {"value": broken}))
            value["targets"][0]["occurrence_scopes"][0]["scope"] = value["truth"][0][
                "scope"
            ]
            extra.append(
                dict(label="unobserved-target-scope", request=q | {"value": value})
            )
        if q["name"] == "StageAdoption" and value["outcome"] == 2:
            broken = deepcopy(value)
            broken["binding"]["read_set"] = []
            extra.append(
                dict(label="omitted-context-read", request=q | {"value": broken})
            )
            value["producer_resource"] = value["isolation"]
            extra.append(dict(label="wrong-producer", request=q | {"value": value}))
    invalid_values = extra + [
        dict(
            label=label,
            request=dict(
                name=name,
                op="encode",
                limits=LIMITS,
                value=evidence_sample(name, 0) | fields,
            ),
        )
        for name, label, fields in cases
    ]
    # All these mutations still have a valid wire shape. Freeze their bytes so
    # each decoder must reject the same semantic fault after deserialization.
    from echo.bridge import from_json
    from echo.runtime import Limits, encode_canonical, encode_protobuf

    invalid_bytes = []
    for row in invalid_values:
        q = row["request"]
        name = q["name"]
        native = from_json(name, q["value"])
        for op, encoder in (
            ("canonical", encode_canonical),
            ("protobuf", encode_protobuf),
        ):
            invalid_bytes.append(
                dict(
                    label=f"{row['label']}-{op}",
                    request=dict(
                        name=name,
                        op=op,
                        limits=LIMITS,
                        hex=encoder(name, native, Limits(**LIMITS)).hex(),
                    ),
                )
            )
    return invalid_values + invalid_bytes


def evidence_legal() -> list[Row]:
    rows: list[Row] = []

    def add(name: str, value: Row) -> None:
        rows.append(dict(op="encode", name=name, limits=LIMITS, value=value))

    loc = evidence_sample("SourceLocation", 1)
    add("SourceLocation", msg(source=loc["source"], path=[]))
    add(
        "SourceLocation",
        msg(
            source=loc["source"],
            path=[msg(field_number=3, element="0"), msg(field_number=2)],
            text=loc["text"],
        ),
    )
    add("SourceField", msg(field_number=4, element=str(2**64 - 1)))
    meta = evidence_sample("SourceMetadataV2", 1)
    original = meta["roots"][0]
    derived = original | {"record_id": "transcript"}
    step = msg(inputs=[original], output=derived, rule=meta["producer"])
    meta.update(transformations=[step], locations=[loc, loc | {"source": derived}])
    layer = msg(
        source=original,
        visible=loc["text"],
        principal_id="speaker",
        identity_evidence=meta["validity"],
    )
    meta["speaker"] = msg(
        layers=[layer, layer | {"source": derived, "principal_id": "quoted"}],
        governance=meta["validity"],
    )
    add("SourceMetadataV2", meta)
    m = evidence_sample("StageAdoption", 2)
    m.update(
        outcome=2,
        reason="one target unresolved",
        coverage=[
            msg(target="known", mu=0.0),
            msg(target="unknown", unknown_reason="insufficient observation"),
        ],
    )
    add("StageAdoption", m)
    failed = m | {"outcome": 3, "reason": "dispatch failed", "coverage": []}
    failed.pop("candidate")
    failed.pop("actual_output")
    add("StageAdoption", failed)
    for source, tag in [
        (1, "echo.user-text"),
        (2, "echo.touch-v2"),
        (3, "synthetic.absence"),
        (6, "echo.internal-activity-record-v2"),
    ]:
        rule = msg(
            source=source,
            event_kind=2,
            payload_type=tag,
            exogenous=source <= 3,
            self_narrative=source == 6,
            growth_access=2 if source <= 3 else 1,
        )
        add("AttributeRule", rule)
    a = evidence_sample("EventAttributesRecordV2", 0)
    old = a["event"]
    e = old | {
        "content": old["content"]
        | {"object_type": "echo.user-text", "byte_length": "32"}
    }
    scope = msg(source=e, path=[msg(field_number=2)], text=msg(start="0", end="3"))
    whole = msg(source=e, path=[])
    speaker = msg(
        source=e,
        visible=scope["text"],
        principal_id="human",
        identity_evidence=meta["validity"],
    )
    a.update(
        event=e,
        inputs=[e],
        exogenous=True,
        c_unit=e,
        directed_to_n=e,
        person_mapping=meta["validity"],
        applied=a["applied"]
        | {"payload_type": "echo.user-text", "source": 1, "exogenous": True},
        source=a["source"]
        | {
            "roots": [e],
            "locations": [scope],
            "speaker": msg(layers=[speaker], governance=meta["validity"]),
        },
        occurred=[msg(scope=whole, value=1, evidence=[e])],
        truth=[msg(scope=scope, value=3, evidence=[], reason="conflicting accounts")],
        subjective=[msg(scope=scope, value=1, evidence=[e], experiencer="human")],
    )
    a["targets"][0]["occurrence_scopes"] = [msg(scope=whole, expected=1)]
    add("EventAttributesRecordV2", a)
    return rows
