"""Experimental lossless IRL wire compression; not a production contract."""
from __future__ import annotations

from copy import deepcopy
import json
from pathlib import Path
import re
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
VERSION = "p0-08-candidate-v1"
KINDS = ["Ref", "Claim", "Act"]
BASE = {"id": "i", "sourceRefs": "f", "sourceSpan": "b", "derivation": "d",
        "epistemicType": "e", "resolutionState": "q", "kind": "k"}
PAYLOAD = {"Ref": {"mention": "m"},
           "Claim": {"predicateLabel": "p", "roles": "r", "value": "v",
                     "polarity": "o", "modality": "m", "time": "t"},
           "Act": {"actType": "a", "content": "c"}}
WIRE = json.loads((ROOT / "spikes/codex-app-server/semantic-candidate.wire.schema.json")
                  .read_text(encoding="utf-8"))
ENUMS = {key: WIRE["$defs"]["claimCandidate"]["properties"][key]["enum"]
         for key in ["derivation", "epistemicType", "resolutionState", "polarity", "modality"]}
ENUMS["actType"] = WIRE["$defs"]["actCandidate"]["properties"]["actType"]["enum"]
ENUMS["kind"] = KINDS


def encode_ref(value: str) -> int | str:
    match = re.fullmatch(r"p0-08-([0-9]{4})", value)
    return int(match[1]) if match else value


def decode_ref(value: int | str) -> str:
    if isinstance(value, int) and not isinstance(value, bool) and 0 <= value <= 9999:
        return f"p0-08-{value:04d}"
    if isinstance(value, str):
        return value
    raise ValueError("Invalid source identifier")


def shrink(candidate: dict[str, Any]) -> dict[str, Any]:
    if set(candidate) != {"schemaVersion", "sourceRef", "nodes"} or candidate["schemaVersion"] != VERSION:
        raise ValueError("Wrong experimental schema version")
    result: dict[str, Any] = {"v": 1, "s": encode_ref(candidate["sourceRef"]), "n": []}
    for node in candidate["nodes"]:
        if node["representationVersion"] != VERSION:
            raise ValueError("Wrong node representation version")
        mapping = BASE | PAYLOAD[node["kind"]]
        if set(node) != set(mapping) | {"representationVersion"}:
            raise ValueError("Unknown/missing fields must not be silently discarded")
        compact = {}
        for key, short in mapping.items():
            value = node[key]
            if key in ENUMS:
                value = ENUMS[key].index(value)
            elif key == "sourceRefs":
                value = [encode_ref(item) for item in value]
            elif key == "roles":
                value = [{"k": role["role"], "v": role["value"]} for role in value]
            compact[short] = value
        result["n"].append(compact)
    return result


def expand(candidate: dict[str, Any]) -> dict[str, Any]:
    if set(candidate) != {"v", "s", "n"} or type(candidate["v"]) is not int or candidate["v"] != 1:
        raise ValueError("Unknown compact root fields/version")
    result: dict[str, Any] = {"schemaVersion": VERSION, "sourceRef": decode_ref(candidate["s"]), "nodes": []}
    for compact in candidate["n"]:
        code = compact["k"]
        if type(code) is not int or not 0 <= code < len(KINDS):
            raise ValueError("Invalid kind code")
        mapping = BASE | PAYLOAD[KINDS[code]]
        if set(compact) != set(mapping.values()):
            raise ValueError("Unknown/missing compact node fields")
        node = {"representationVersion": VERSION}
        for key, short in mapping.items():
            value = compact[short]
            if key in ENUMS:
                if type(value) is not int or not 0 <= value < len(ENUMS[key]):
                    raise ValueError("Invalid enum code")
                value = ENUMS[key][value]
            elif key == "sourceRefs":
                value = [decode_ref(item) for item in value]
            elif key == "roles":
                if any(set(role) != {"k", "v"} for role in value):
                    raise ValueError("Unknown role fields")
                value = [{"role": role["k"], "value": role["v"]} for role in value]
            node[key] = value
        result["nodes"].append(node)
    return result


def compact_schema() -> dict[str, Any]:
    definitions = {}
    reference = {"anyOf": [{"type": "integer", "minimum": 0, "maximum": 9999}, {"type": "string"}]}
    for index, kind in enumerate(KINDS):
        original = WIRE["$defs"][kind.lower() + "Candidate"]
        mapping = BASE | PAYLOAD[kind]
        properties = {}
        for key, short in mapping.items():
            value = deepcopy(original["properties"][key])
            if key == "kind":
                value = {"const": index}
            elif key in ENUMS:
                value = {"enum": list(range(len(ENUMS[key])))}
            elif key == "sourceRefs":
                value = {"type": "array", "minItems": 1, "uniqueItems": True, "items": reference}
            elif key == "roles":
                value = {"type": "array", "items": {"type": "object", "additionalProperties": False,
                         "required": ["k", "v"], "properties": {"k": {"type": "string"}, "v": {"type": "string"}}}}
            properties[short] = value
        definitions[f"n{index}"] = {"type": "object", "additionalProperties": False,
                                    "required": list(properties), "properties": properties}
    return {"type": "object", "additionalProperties": False, "required": ["v", "s", "n"],
            "properties": {"v": {"const": 1}, "s": {"type": "integer", "minimum": 0, "maximum": 9999},
                           "n": {"type": "array", "items": {"anyOf": [{"$ref": f"#/$defs/n{i}"} for i in range(3)]}}},
            "$defs": definitions}


def legend() -> dict[str, Any]:
    return {"version": 1, "fullVersion": VERSION, "root": {"s": "sourceRef p0-08-NNNN as integer", "n": "nodes"},
            "metadata": BASE, "payloadByKind": PAYLOAD, "enumIndexZeroBased": ENUMS,
            "sourceRefs": "p0-08-NNNN sources use integers; other sources remain strings",
            "roles": "r items: k=role, v=value; preserve strings and node IDs",
            "omittedConstant": "schemaVersion and representationVersion expand to fullVersion"}
