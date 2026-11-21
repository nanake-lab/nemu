"""ADR 0047. Check decoded bindings before consulting the trusted issuing service."""

from __future__ import annotations

from hashlib import sha256
from typing import Any

from .speak_think import check_speak_think_message
from .runtime import SCHEMA, Limits, fail, normalize


def check_permission_message(name: str, value: dict[str, Any], limits: Limits) -> None:
    if SCHEMA["messages"].get(name, {}).get("file") == "permission.proto":
        v = normalize(name, value, limits)
        if name == "PermissionContext":
            if "remote_text" in v:
                check_speak_think_message(
                    "RemoteCognitiveReceipt", v["remote_text"], limits
                )
            else:
                _check("ExecutionCredential", v["local_execution"])
        else:
            _check(name, v)


def _check(name: str, v: dict[str, Any]) -> None:
    if name == "PermissionResult" and "management_commit" in v:
        fail(v["management_commit"]["kind"] in (1, 2, 3), "invalid maintenance result")
    elif name == "TerminalResponseV2":
        m = v.get("management")
        fail(
            (v["reply_to"] == 6) == (m is not None),
            "missing or unexpected management reply",
        )
        if m is not None:
            _check("PermissionResult", m)
            fail(
                m["request_id"] == v["request_id"]
                and (
                    (v["kind"] == 4 and "management_commit" in m)
                    or (v["kind"] == 5 and "denied" in m)
                ),
                "wrong management reply",
            )
    elif name == "ExecutionBudget":
        fail(v["charge"] <= v["limit"], "charge exceeds grant")
    elif name == "ConnectorScope":
        if "wallet_minor_units" in v:
            fail(v["wallet_minor_units"] > 0, "empty wallet transfer")
    elif name == "TrustMemory":
        fail(v["tier"] in (2, 3), "only T1/T2 trust")
        _check("ConnectorScope", v["scope"])
    elif name == "GrowthCommandV2" and v["kind"] == 3:
        fail(
            v["alarm_record"]["object_type"] == "echo.system-maintenance-record",
            "wrong alarm record",
        )
        fail(("replacement_law" in v) == (v["verdict"] == 2), "replacement law")
        fail(v.get("replacement_law") != v["law_version"], "law unchanged")
    elif name == "GrowthManagementRequest":
        _check("GrowthCommandV2", v["command"])
        fail(v["entry"] == 2 or v["command"]["kind"] != 3, "review requires echoctl")
    elif name == "TerminalRequestV2" and v["kind"] == 6:
        management = v["management"]
        _check("GrowthManagementRequest", management)
        fail(v["session_id"] == management["user"]["session_id"], "wrong session")
    elif name == "ExecutionCredential":
        scope, tier = v["scope"], v["tier"]
        _check("ConnectorScope", scope)
        _check("ExecutionBudget", v["budget"])
        fail(v["issued_at_ns"] < v["expires_at_ns"], "empty validity interval")
        trust, consent = v.get("trust"), v.get("consent")
        if trust is not None:
            _check("TrustMemory", trust)
            fail(
                trust["scope"] == scope
                and trust["tier"] == tier
                and trust["revocation_version"] == v["revocation_version"],
                "trust binding",
            )
        if consent is not None:
            fail(
                consent["action_id"] == v["action_id"]
                and consent["payload_digest"] == v["payload_digest"]
                and consent["confirmed_at_ns"] <= v["issued_at_ns"],
                "consent binding",
            )
        if scope["kind"] == 2:
            if not scope["reversible"]:
                policy = scope["desktop"]["irreversible"]
                fail(
                    policy == 3 or (policy == 2 and consent is not None),
                    "desktop policy",
                )
        elif tier == 4 or (scope["kind"] == 1 and tier in (2, 3) and trust is None):
            fail(consent is not None, "missing consent")
    elif name == "ActionRequestV2":
        grant = v["grant"]
        _check("ExecutionCredential", grant)
        fail(
            v["action_id"] == grant["action_id"]
            and v["expires_at_ns"] == grant["expires_at_ns"]
            and sha256(v["action_payload"]).digest() == grant["payload_digest"],
            "dispatch binding",
        )
