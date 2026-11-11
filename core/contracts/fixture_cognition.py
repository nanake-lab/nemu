"""ADR 0030 / ADR 0026 §§3–6: synthetic contracts, never real H/Q."""

from __future__ import annotations

import copy
from typing import Any
from fixture_law_state import ref

NAMES = {
    "CognitionBinding",
    "CognitiveMaterial",
    "CognitiveReturnedResult",
    "CognitiveInput",
    "CognitiveResultAndReceipt",
    "InternalActivityRecord",
    "ActivityState",
    "CognitiveInflightAction",
    "CognitionState",
    "InternalActivityCommit",
    "CoreTransactionV3",
    "SystemMaintenanceRecord",
}


def sample_cognition(name: str, seed: int) -> dict[str, Any]:
    from fixture_inputs import sample

    i = seed % 3
    b: dict[str, Any] = dict(
        schema_version=1,
        activity_id=f"activity-{i}",
        step_id=f"step-{i}",
        request_id=f"request-{i}",
        round_index=str(2**53 + i),
        branch_id="synthetic",
        base_commit=f"commit-{i}",
        q_revision=f"q-{i}",
        input_frontier=[f"event-{i}"],
        snapshot_id=f"snapshot-{i}",
        environment=sample("VersionedEnvironmentV2", i),
        protocol=ref("echo.protocol", 1),
        permission_snapshot=ref("echo.permissions", 2),
        source_validity_version=f"sources-{i}",
        read_set=[ref("echo.record", 3)],
    )
    if name == "CognitionBinding":
        return b
    if name == "CognitiveMaterial":
        return dict(
            schema_version=1,
            content=ref("echo.content", i + 1),
            metadata=sample("SourceMetadata", i),
            past_reaction=i == 2,
        )
    if name == "CognitiveReturnedResult":
        v = dict(schema_version=1, request_id=b["request_id"], status=i + 1)
        if i == 1:
            v["result"] = ref("echo.result", 4)
        return v
    if name == "CognitiveInput":
        return dict(
            schema_version=1,
            binding=b,
            materials=[sample_cognition("CognitiveMaterial", i)],
            new_inputs=[],
            results=[sample_cognition("CognitiveReturnedResult", i)],
            retrievals=[],
            budget=ref("echo.budget", 5),
        )
    if name == "CognitiveResultAndReceipt":
        return dict(
            schema_version=1,
            binding=b,
            input=ref("echo.cognitive-input", 6),
            products=[sample_cognition("CognitiveMaterial", i)],
            queries=[],
            completion=1,
            model_hash=b["environment"]["model"]["digest"],
            tokenizer_hash=b["environment"]["tokenizer"]["digest"],
            modulation=ref("echo.modulation-v2", 7),
            carrier_receipt=ref("echo.carrier-receipt", 8),
            usage=ref("echo.usage", 9),
        )
    if name == "InternalActivityRecord":
        return dict(
            schema_version=1,
            record_id=f"record-{i}",
            result=sample_cognition("CognitiveResultAndReceipt", i),
            started_at_ns=str(2**53 + i),
            finished_at_ns=str(2**53 + 100 + i),
        )
    if name == "ActivityState":
        return dict(
            schema_version=1,
            activity_id=b["activity_id"],
            revision=b["q_revision"],
            materials=[ref("echo.content", 1)],
            waiting=[],
            status=i + 1,
            latest_step=ref("echo.internal-activity-record", 2),
        )
    if name == "CognitiveInflightAction":
        return dict(
            schema_version=1,
            action_id=f"action-{i}",
            origin_record=ref("echo.internal-activity-record", 2),
            origin_step_id=b["step_id"],
            round_index=b["round_index"],
            registered_at_ns=str(2**53 + i),
            plan=sample("ActionPlan", i),
            status=i + 1,
            request=ref("echo.action-request", 10),
        )
    if name == "CognitionState":
        v = dict(
            schema_version=1,
            revision=b["q_revision"],
            mode=i + 1,
            activities=[sample_cognition("ActivityState", i)],
            inflight=[sample_cognition("CognitiveInflightAction", i)],
            transition=ref("echo.system-maintenance-record", 11),
        )
        if i == 0:
            v["active_request"] = ref("echo.cognitive-input", 6)
        else:
            v["reason"] = "explicit synthetic transition"
        return v
    if name == "InternalActivityCommit":
        return dict(
            schema_version=1,
            binding=b,
            record=sample_cognition("InternalActivityRecord", i),
            interpretations=[ref("echo.semantic-record-v2", 12)],
        )
    if name == "CoreTransactionV3":
        v = sample("CoreTransactionV2", i)
        binding = v.pop("binding")
        if i == 0:
            v["event_binding"] = binding
        else:
            v["internal_commit"] = sample_cognition("InternalActivityCommit", i)
        return v
    if name == "SystemMaintenanceRecord":
        return dict(
            schema_version=1,
            record_id=f"maintenance-{i}",
            kind=i + 1,
            reason="explicit maintenance",
            details=ref("echo.maintenance-details", 13),
            ledger_refs=[],
            related_commit=b["base_commit"],
            recorded_at_ns=str(2**53 + i),
        )
    raise ValueError(name)


def legal_cognition() -> list[dict[str, Any]]:
    from fixture_inputs import request, sample

    rows = []
    # ADR 0026 §5.2: distinguish every returned outcome; no implicit failure/success.
    for status in range(1, 10):
        v = sample_cognition("CognitiveReturnedResult", 0)
        v["status"] = status
        if status == 2:
            v["result"] = ref("echo.result", 4)
        rows.append(request("CognitiveReturnedResult", v))
    # §6.7: retain actual partial products; never publish an incomplete successor.
    for completion in (2, 3, 4):
        v = sample_cognition("CognitiveResultAndReceipt", 0)
        v.update(completion=completion, reason="synthetic interruption")
        if completion != 2:
            v.pop("carrier_receipt")
        rows.append(request("CognitiveResultAndReceipt", v))
    # §§3,4: no-op, full plan, and RUNNING with only a registered successor.
    v = sample_cognition("CognitiveResultAndReceipt", 0)
    v["products"] = []
    p = sample("ActionPlan", 0)
    p.update(
        snapshot_id=v["binding"]["snapshot_id"],
        task_version=v["binding"]["q_revision"],
        protocol_version=v["binding"]["protocol"]["resource_version"],
    )
    v["action_plan"] = p
    rows.append(request("CognitiveResultAndReceipt", v))
    v = sample_cognition("CognitionState", 0)
    v["runnable_successor"] = v.pop("active_request")
    rows.append(request("CognitionState", v))
    # V2-87/102: all maintenance categories have an explicit, separate record.
    for kind in (4, 5):
        v = sample_cognition("SystemMaintenanceRecord", 0)
        v["kind"] = kind
        rows.append(request("SystemMaintenanceRecord", v))
    return rows


def illegal_cognition() -> list[dict[str, Any]]:
    from fixture_inputs import request, LIMITS, sample
    from echo.bridge import from_json
    from echo.runtime import Limits, encode_canonical, encode_protobuf

    rows: list[dict[str, Any]] = []

    def add(name: str, v: dict[str, Any], label: str, wire: bool = True) -> None:
        rows.append(dict(label="P5-37-" + label, request=request(name, v)))
        if wire:
            for op, encode in (
                ("canonical", encode_canonical),
                ("protobuf", encode_protobuf),
            ):
                raw = encode(name, from_json(name, v), Limits(**LIMITS))
                rows.append(
                    dict(
                        label="P5-37-" + label,
                        request=dict(name=name, op=op, hex=raw.hex(), limits=LIMITS),
                    )
                )

    # ADR 0026 §6.5/6.6: every binding component, not just request_id, must agree.
    for key in sample_cognition("CognitionBinding", 0):
        if key == "schema_version":
            continue
        v = sample_cognition("InternalActivityCommit", 0)
        other = sample_cognition("CognitionBinding", 1)[key]
        if other == v["binding"][key]:
            if isinstance(other, dict):
                other = copy.deepcopy(other)
                other["digest"] = "ff" * 32
            elif isinstance(other, list):
                other = []
            else:
                other = "different"
        v["binding"][key] = other
        add("InternalActivityCommit", v, "binding-" + key)
    # §6.7, §5, §3: semantic failures are rejected after either wire decoder too.
    mutations = [
        ("CognitiveReturnedResult", "status", 2),
        ("CognitiveReturnedResult", "result", ref("echo.result", 1)),
        ("CognitiveResultAndReceipt", "input", ref("echo.output-chunk", 1)),
        ("CognitiveResultAndReceipt", "modulation", ref("echo.content", 1)),
        ("CognitiveResultAndReceipt", "model_hash", "ff" * 32),
        ("CognitiveResultAndReceipt", "tokenizer_hash", "ff" * 32),
        ("CognitiveResultAndReceipt", "carrier_receipt", None),
        ("CognitiveResultAndReceipt", "completion", 3),
        ("InternalActivityRecord", "finished_at_ns", "0"),
        ("InternalActivityRecord", "previous", ref("echo.output-chunk", 1)),
        ("ActivityState", "latest_step", ref("echo.output-chunk", 1)),
        ("ActivityState", "successor", ref("echo.output-chunk", 1)),
        ("CognitiveInflightAction", "origin_record", ref("echo.output-chunk", 1)),
        ("CognitionState", "active_request", None),
        ("CognitionState", "runnable_successor", ref("echo.cognitive-input", 1)),
        ("CognitionState", "transition", ref("echo.output-chunk", 1)),
        ("InternalActivityCommit", "interpretations", []),
    ]
    for name, key, value in mutations:
        v = sample_cognition(name, 0)
        if value is None:
            v.pop(key)
        else:
            v[key] = value
        add(name, v, key)
    for mode in (2, 3):
        v = sample_cognition("CognitionState", mode - 1)
        v.pop("reason")
        add("CognitionState", v, "mode-reason")
        v["reason"] = "explicit"
        v["active_request"] = ref("echo.cognitive-input", 1)
        add("CognitionState", v, "paused-active")
    for key in ("next_focus", "queries", "action_plan"):
        v = sample_cognition("CognitiveResultAndReceipt", 0)
        v.update(completion=2, reason="truncated")
        v[key] = (
            sample("ActionPlan", 0)
            if key == "action_plan"
            else ([ref("echo.query", 1)] if key == "queries" else ref("echo.focus", 1))
        )
        add("CognitiveResultAndReceipt", v, "incomplete-" + key)
    for key in ("snapshot_id", "task_version", "protocol_version"):
        v = next(
            copy.deepcopy(r["value"])
            for r in legal_cognition()
            if "action_plan" in r["value"]
        )
        v["action_plan"][key] = "stale"
        add("CognitiveResultAndReceipt", v, "plan-" + key)
    # §6.1: exactly one origin; recurse through transaction to the nested result.
    v = sample_cognition("CoreTransactionV3", 1)
    v["internal_commit"]["record"]["result"]["model_hash"] = "ff" * 32
    add("CoreTransactionV3", v, "nested-receipt")
    for both in (False, True):
        v = sample_cognition("CoreTransactionV3", 0)
        if both:
            v["internal_commit"] = sample_cognition("InternalActivityCommit", 0)
        else:
            v.pop("event_binding")
        add("CoreTransactionV3", v, "origin-oneof", wire=False)
    return rows
