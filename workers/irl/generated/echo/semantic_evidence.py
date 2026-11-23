"""ADR 0048 checks on normalized candidate records, before trusted resolution."""

from __future__ import annotations

from typing import Any
from .runtime import Limits, SCHEMA, fail, normalize
from .speak_think import check_speak_think_message

Row = dict[str, Any]


def _registry(v: Row) -> None:
    ids = {e["concept_id"] for e in v["entries"]}

    def member(c: Row) -> bool:
        return c["concept_id"] in ids and c["registry_version"] == v["version"]

    for e in v["entries"]:
        fail(
            all(member(r["domain"]) for r in e["roles"] if "domain" in r), "role domain"
        )
    for link in v["links"] + v["aliases"]:
        fail(member(link["origin"]) and member(link["target"]), "unknown concept")
    fail(all(e["kind"] in v["relation_kinds"] for e in v["links"]), "unknown relation")
    edges: dict[str, str] = {}
    for a in v["aliases"]:
        key = a["origin"]["concept_id"]
        fail(a["kind"] == "canonical" and key not in edges, "ambiguous alias")
        edges[key] = a["target"]["concept_id"]
    done: set[str] = set()
    for start in edges:
        path: set[str] = set()
        while start in edges and start not in done:
            fail(start not in path, "alias cycle")
            path.add(start)
            start = edges[start]
        done.update(path)


def _stage(v: Row) -> None:
    b, stage, outcome = v["binding"], v["stage"], v["outcome"]
    fail(
        v["event"]["branch_id"] == b["branch_id"] == v["interpretation"]["branch_id"]
        and v["interpretation"] != v["event"],
        "interpretation endpoints",
    )
    fail(
        v["registry_before"] == b["environment"]["registry"]
        and v["context"]["digest"] == b["c_core_digest"]
        and all(d in b["read_set"] for d in [v["context"], *v["definitions"]]),
        "stage read set",
    )
    resource = b["environment"].get(
        ["irl", "nature_appraiser", "situation_matcher"][stage - 1]
    )
    fail(
        resource == v.get("producer_resource")
        and (resource is not None or outcome == 3),
        "stage producer",
    )
    fail(stage == 3 or not v["definitions"], "definitions outside M")
    fail(
        ("registry_after" in v) == bool(v["staged_revisions"])
        and ("registry_after" not in v or stage == 1),
        "registry change",
    )
    fail(("reason" in v) == (outcome != 1), "stage reason")
    if outcome == 3:
        fail(
            not ({"appraisal", "registry_after", "revision"} & v.keys())
            and not v["coverage"],
            "failed stage",
        )
    else:
        fail("candidate" in v and "actual_output" in v, "stage inputs")
        if stage == 2:
            fail(outcome == 1 and "appraisal" in v and not v["coverage"], "invalid A")
        else:
            fail(
                "appraisal" not in v and (outcome != 2 or bool(v["coverage"])),
                "partial stage",
            )
            fail(
                all(
                    ("mu" if stage == 1 else "accepted") not in c for c in v["coverage"]
                ),
                "coverage stage",
            )
    edge = v.get("revision")
    fail(bool(edge) == bool(v["replaced_targets"]), "replacement targets")
    if edge:
        fail(
            edge["kind"] == 14
            and edge["origin"] == v["interpretation"]
            and edge["target"] != edge["origin"]
            and edge["target"]["branch_id"] == b["branch_id"],
            "revision endpoints",
        )
    if "relations" in v:
        q = v["relations"]["result"]
        fail(
            q["endpoint"] == (edge["target"] if edge else v["event"])
            and q["frontier"] == b["base_commit"]
            and (not edge or edge["purpose"] == q["purpose"]),
            "relation scope",
        )


def _rule(v: Row) -> None:
    s = v["source"]
    fail(
        (v["exogenous"], v["self_narrative"], v["growth_access"])
        == (s <= 3, s == 6, [2, 2, 2, 2, 3, 1, 1][s - 1]),
        "source partition",
    )
    types = {
        "echo.user-text": 1,
        "echo.user-utterance": 1,
        "echo.touch-v2": 2,
        "echo.desktop-observation": 4,
        "echo.body-action-result": 5,
        "echo.terminal-maintenance": 7,
        "echo.system-maintenance-record": 7,
        "echo.internal-activity-record": 6,
        "echo.internal-activity-record-v2": 6,
    }
    fail(
        types.get(v["payload_type"], s) == s
        and (v["event_kind"] != 8 or s == 7)
        and (v["event_kind"] != 4 or s == 5),
        "event source",
    )


def _attributes(v: Row) -> None:
    r, e, inputs = v["applied"], v["event"], v["inputs"]
    fail(
        v["event_id"] == e["record_id"]
        and r["payload_type"] == e["content"]["object_type"]
        and v["rule_table"]["object_type"] == "echo.attribute-rule-table"
        and v["rule_table_version"] == v["rule_table"]["resource_version"]
        and v["exogenous"] == r["exogenous"]
        and v["self_narrative"] == r["self_narrative"]
        and ("c_unit" in v) == r["exogenous"],
        "attribute rule",
    )
    fail(
        e in inputs
        and all(i["branch_id"] == e["branch_id"] for i in inputs)
        and any(loc["source"] == e for loc in v["source"]["locations"]),
        "event inputs",
    )
    fail(
        all(v[k] in inputs for k in ("c_unit", "directed_to_n", "cause") if k in v),
        "classification evidence",
    )
    s = r["source"]
    if s in (1, 2):
        fail(
            "directed_to_n" in v
            and "person_mapping" in v
            and (s != 1 or "speaker" in v["source"]),
            "unverified actor",
        )
    fail(
        (s != 5 or "cause" in v) and (s not in (2, 3) or "episode_phase_fact" in v),
        "missing origin",
    )
    for axis, requirements in [
        ("occurred", "occurrence_scopes"),
        ("truth", "truth_scopes"),
        ("subjective", "subjective_scopes"),
    ]:
        items = v[axis]
        fail(bool(items), "unassessed axis")
        for item in items:
            fail(
                item["scope"]["source"] in inputs
                and all(i in inputs for i in item["evidence"]),
                "verification evidence",
            )
            if axis == "subjective":
                fail(item["value"] != 1 or "experiencer" in item, "missing experiencer")
            else:
                fail("experiencer" not in item, "unexpected experiencer")
        for t in v["targets"]:
            fail(
                all(
                    any(i["scope"] == p["scope"] for i in items)
                    for p in t[requirements]
                ),
                "unobserved premise",
            )


def check_semantic_evidence(name: str, value: Row, limits: Limits) -> None:
    meta = SCHEMA["messages"].get(name, {})
    if meta.get("file") != "semantic_evidence.proto":
        return
    v = normalize(name, value, limits)
    for f in meta["fields"]:
        t = f["type"]
        if f["name"] not in v or t not in SCHEMA["messages"]:
            continue
        child = v[f["name"]]
        for item in child if f["repeated"] else [child]:
            check_speak_think_message(t, item, limits)
            check_semantic_evidence(t, item, limits)
    if name == "SourceLocation" and "text" in v:
        fail(v["text"]["start"] < v["text"]["end"], "empty range")
    elif name == "SourceMetadataV2":
        seen = list(v["roots"])
        fail(
            bool(seen)
            and bool(v["locations"])
            and all(r["branch_id"] == seen[0]["branch_id"] for r in seen),
            "missing or cross-branch roots",
        )
        for step in v["transformations"]:
            fail(
                bool(step["inputs"])
                and all(r in seen for r in step["inputs"])
                and step["output"] not in seen
                and step["output"]["branch_id"] == seen[0]["branch_id"],
                "transform chain",
            )
            seen.append(step["output"])
        fail(all(loc["source"] in seen for loc in v["locations"]), "location binding")
        if "speaker" in v:
            fail(
                any(
                    loc["source"] == v["speaker"]["layers"][0]["source"]
                    for loc in v["locations"]
                ),
                "speaker source",
            )
    elif name == "ConceptRevision" and v["lifecycle"] == 2:
        fail(
            "label" in v and "definition" in v and bool(v["grounding"]),
            "stable grounding",
        )
    elif name == "ConceptRegistry":
        _registry(v)
    elif name == "ClaimV2":
        fail(
            "text" in v["reported_text"]
            and v["reported_text"] in v["source"]["locations"]
            and ("time_fact" in v) == (v["time_kind"] == 3),
            "claim operand",
        )
    elif name == "SemanticGraphV2":
        nodes = [n["id"] for k in ("references", "statements", "actions") for n in v[k]]
        fail(
            len(set(nodes)) == len(nodes)
            and all(
                e["from_node"] in nodes and e["to_node"] in nodes for e in v["edges"]
            )
            and all(
                r["node_ref"] in nodes for c in v["statements"] for r in c["roles"]
            ),
            "graph nodes",
        )
    elif name == "TextAdapterEntry":
        fail(
            v["payload_tag"] in ("echo.user-text", "echo.user-utterance")
            and v["payload_schema"] == 1
            and v["path"] == [{"schema_version": 1, "field_number": 2}],
            "unsupported adapter",
        )
    elif name == "TextAdapterTable":
        fail(
            v["registry"]["object_type"] == "echo.concept-registry"
            and len(v["entries"]) == 2
            and all(
                e["predicate"]["registry_version"] == v["registry"]["resource_version"]
                for e in v["entries"]
            ),
            "adapter table",
        )
    elif name == "StageCoverage":
        fail(
            len({"accepted", "mu", "unknown_reason"} & v.keys()) == 1,
            "ambiguous coverage",
        )
    elif name == "StageAdoption":
        _stage(v)
    elif name == "AttributeRule":
        _rule(v)
    elif name == "AttributeRuleTable":
        keys = {(r["source"], r["event_kind"], r["payload_type"]) for r in v["rules"]}
        fail(len(keys) == len(v["rules"]) and bool(keys), "rule keys")
    elif name == "ScopedVerification":
        fail(
            "reason" in v if v["value"] == 3 else bool(v["evidence"]),
            "verification basis",
        )
    elif name == "VerificationRequirement":
        fail(v["expected"] != 3, "unknown prerequisite")
    elif name == "TargetPremise":
        fail(
            any(
                v[k] for k in ("occurrence_scopes", "truth_scopes", "subjective_scopes")
            ),
            "empty premise",
        )
    elif name == "EventAttributesRecordV2":
        _attributes(v)
