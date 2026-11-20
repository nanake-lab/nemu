"""ADR 0030 boundary: synthetic/structural validity does not confer commit authority.

Call after decoding, before consuming. Reference resolution, source/permission
closure, real carrier verification and current-Q comparison belong to the caller.
"""

from __future__ import annotations
from typing import Any
from .runtime import Limits, encode_canonical, fail, normalize

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


def check_cognition_message(name: str, value: dict[str, Any], limits: Limits) -> None:
    if name not in NAMES:
        return
    v = normalize(name, value, limits)

    def tag(key: str, expected: str) -> None:
        if key in v:
            fail(v[key]["object_type"] == expected, f"{name}.{key} type")

    def child(key: str, child_name: str) -> None:
        if key in v:
            check_cognition_message(child_name, v[key], limits)

    if name == "CognitiveReturnedResult":
        fail(v["status"] != 2 or "result" in v, "successful result missing")
        fail(v["status"] != 1 or "result" not in v, "waiting has result")
    elif name == "CognitiveInput":
        for item in v["results"]:
            check_cognition_message("CognitiveReturnedResult", item, limits)
    elif name == "CognitiveResultAndReceipt":
        tag("input", "echo.cognitive-input")
        tag("modulation", "echo.modulation-v2")
        b = v["binding"]
        fail(v["model_hash"] == b["environment"]["model"]["digest"], "model binding")
        fail(
            v["tokenizer_hash"] == b["environment"]["tokenizer"]["digest"],
            "tokenizer binding",
        )
        fail(
            v["completion"] not in (1, 2) or "carrier_receipt" in v,
            "carrier receipt missing",
        )
        if v["completion"] != 1:
            fail("reason" in v, "incomplete reason missing")
            fail(
                "next_focus" not in v and "action_plan" not in v and not v["queries"],
                "incomplete successor",
            )
        if "action_plan" in v:
            p = v["action_plan"]
            fail(p["snapshot_id"] == b["snapshot_id"], "action snapshot")
            fail(p["task_version"] == b["q_revision"], "action Q revision")
            fail(
                p["protocol_version"] == b["protocol"]["resource_version"],
                "action protocol",
            )
    elif name == "InternalActivityRecord":
        tag("previous", "echo.internal-activity-record")
        fail(v["finished_at_ns"] >= v["started_at_ns"], "request execution interval")
        child("result", "CognitiveResultAndReceipt")
    elif name == "ActivityState":
        tag("latest_step", "echo.internal-activity-record")
        tag("successor", "echo.cognitive-input")
    elif name == "CognitiveInflightAction":
        tag("origin_record", "echo.internal-activity-record")
    elif name == "CognitionState":
        tag("active_request", "echo.cognitive-input")
        tag("runnable_successor", "echo.cognitive-input")
        tag("transition", "echo.system-maintenance-record")
        count = int("active_request" in v) + int("runnable_successor" in v)
        fail(
            count == 1 if v["mode"] == 1 else count == 0 and "reason" in v,
            "global continuation",
        )
        for item in v["activities"]:
            check_cognition_message("ActivityState", item, limits)
        for item in v["inflight"]:
            check_cognition_message("CognitiveInflightAction", item, limits)
    elif name == "InternalActivityCommit":
        child("record", "InternalActivityRecord")
        fail(bool(v["interpretations"]), "accepted interpretation outcomes missing")
        fail(
            encode_canonical("CognitionBinding", v["binding"], limits)
            == encode_canonical(
                "CognitionBinding", v["record"]["result"]["binding"], limits
            ),
            "internal binding",
        )
    elif name == "CoreTransactionV3":
        child("internal_commit", "InternalActivityCommit")
