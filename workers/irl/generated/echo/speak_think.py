"""ADR 0046 local consistency checks. Resolve evidence against trusted H before use."""

from __future__ import annotations

from typing import Any
from .cognition import check_cognition_message
from .runtime import SCHEMA, Limits, digest, encode_canonical, fail, normalize


def interval(value: dict[str, Any], length: int) -> tuple[int, int]:
    start, end = value["start"], value["end"]
    fail(0 <= start < end <= length, "invalid UTF-8 range")
    return start, end


def _check_remote(v: dict[str, Any]) -> None:
    for key in ("model_sha256", "tokenizer_sha256", "request_sha256"):
        fail(key not in v or any(v[key]), "unavailable digest encoded as zero")
    if v["status"] != 6:
        fail(v["output_reserve"] <= v["capacity"], "output reserve exceeds capacity")
    fail(
        "raw_response" in v or not ({"returned_text", "returned_model"} & v.keys()),
        "missing response",
    )
    adopted = v["adoption"] in (1, 2)
    fail(
        ("product" in v) == adopted and ("accepted_range" in v) == adopted,
        "adoption payload",
    )
    fail(v["adoption"] == 5 or "review" in v, "missing adoption decision")
    length = v.get("returned_text", {}).get("byte_length", 0)
    if adopted:
        start, end = interval(v["accepted_range"], length)
        full = start == 0 and end == length
        product = v["product"]["content"]
        fail(v["adoption"] != 1 or full, "incomplete full adoption")
        fail(product["byte_length"] == end - start, "product length")
        fail(not full or product == v["returned_text"], "full text identity")
    fail(v["adoption"] != 5 or length == 0, "unaccounted returned text")


def check_speak_think_message(name: str, value: dict[str, Any], limits: Limits) -> None:
    if SCHEMA["messages"].get(name, {}).get("file") != "speak_think.proto":
        return
    _check(name, normalize(name, value, limits), limits)


def _check(name: str, v: dict[str, Any], limits: Limits) -> None:
    if name == "Utf8Range":
        interval(v, 2**64 - 1)
    elif name == "QuoteLayer":
        interval(v["visible"], v["source"]["content"]["byte_length"])
    elif name == "SpeakerAttribution":
        layers = v["layers"]
        fail(bool(layers), "missing speaker layers")
        for layer in layers:
            fail(
                layer["source"]["branch_id"] == layers[0]["source"]["branch_id"],
                "quote branch",
            )
            _check("QuoteLayer", layer, limits)
    elif name == "RecordRelation":
        fail(v["origin"]["branch_id"] == v["target"]["branch_id"], "relation branch")
    elif name == "RelationSet":
        seen: set[bytes] = set()
        for edge in v["relations"]:
            _check("RecordRelation", edge, limits)
            fail(
                edge["purpose"] == v["purpose"]
                and v["endpoint"] in (edge["origin"], edge["target"]),
                "relation query scope",
            )
            encoded = encode_canonical("RecordRelation", edge, limits)
            fail(encoded not in seen, "duplicate relation")
            seen.add(encoded)
    elif name == "RelationQueryReceipt":
        result = v["result"]
        _check("RelationSet", result, limits)
        manifest = v["manifest"]
        fail(
            v["expected_count"] == len(result["relations"]), "incomplete relation count"
        )
        fail(
            manifest["object_type"] == "echo.relation-set"
            and manifest["digest"].hex() == digest("RelationSet", result, limits)
            and manifest["byte_length"]
            == len(encode_canonical("RelationSet", result, limits)),
            "relation manifest",
        )
    elif name == "RemoteCognitiveReceipt":
        _check_remote(v)
    elif name == "InternalActivityRecordV2":
        fail(v["finished_at_ns"] >= v["started_at_ns"], "execution interval")
        if "previous" in v:
            fail(
                v["previous"]["object_type"]
                in (
                    "echo.internal-activity-record",
                    "echo.internal-activity-record-v2",
                ),
                "previous record type",
            )
        if "local" in v:
            check_cognition_message("CognitiveResultAndReceipt", v["local"], limits)
        else:
            _check_remote(v["remote"])
    elif name == "InternalActivityCommitV2":
        activity = v["activity"]
        _check("InternalActivityRecordV2", activity, limits)
        result = activity["local"] if "local" in activity else activity["remote"]
        fail(
            result["binding"] == v["binding"] and bool(v["interpretations"]),
            "commit binding or interpretations",
        )
    elif name == "CoreTransactionV4" and "activity_commit" in v:
        _check("InternalActivityCommitV2", v["activity_commit"], limits)
    elif name == "SpeechDeliveryV2":
        length = v["presentation"]["content"]["byte_length"]
        spans = v["delivered"]
        previous = 0
        for i, span in enumerate(spans):
            start, end = interval(span, length)
            fail(i == 0 or previous < start, "delivery ranges overlap or touch")
            previous = end
        full = len(spans) == 1 and spans[0]["start"] == 0 and previous == length
        if v["status"] == 1:
            fail(full or (length == 0 and not spans), "full delivery range")
        elif v["status"] == 2:
            fail(bool(spans) and not full, "partial delivery range")
        elif v["status"] == 4:
            fail(not spans, "unknown delivery claims text")
