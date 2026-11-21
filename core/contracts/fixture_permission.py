"""Synthetic local bindings for ADR 0047. Nothing here creates authority."""

from __future__ import annotations

import hashlib
from typing import Any

from fixture_law_state import ref


def permission_sample(name: str, seed: int) -> dict[str, Any]:
    from fixture_inputs import sample

    if name == "PermissionContext":
        from fixture_speak_think import receipt

        if seed % 2:
            return dict(
                schema_version=1,
                local_execution=permission_sample("ExecutionCredential", seed),
            )
        remote = receipt(seed)
        if seed:
            remote["local_pause_reason"] = "local path unavailable"
        return dict(schema_version=1, remote_text=remote)

    i = seed
    version = dict(schema_version=1)
    payload = f"execute-{i}🙂".encode()
    digest = hashlib.sha256(payload).hexdigest()
    user = dict(
        **version,
        principal_id=f"person-{i}",
        session_id=f"local-{i}",
        installation_id=f"installation-{i}",
        authentication_record_id=f"auth-{i}",
    )
    policy = dict(
        **version,
        version=f"policy-{i}",
        irreversible=1 + i % 3,
        reversibility_rule=ref("fixture.reversibility", 1),
    )
    scope = dict(
        **version,
        connector_id="fixture-connector",
        operation_id=f"operation-{i}",
        resource_id=f"target-{i}",
        kind=2,
        desktop=policy,
        reversible=i % 3 == 0,
    )
    consent = dict(
        **version,
        user=user,
        confirmed_at_ns="9",
        approval_id=f"approval-{i}",
        action_id=f"action-{i}",
        payload_digest=digest,
    )
    budget = dict(
        **version,
        budget_id="calls",
        version=f"period-{i}",
        unit="call",
        limit="4",
        charge="1",
    )
    trust = dict(
        **version,
        version=f"trust-{i}",
        user=user,
        scope=scope,
        tier=3,
        revocation_version="2",
    )
    grant = dict(
        **version,
        issuance_id=f"grant-{i}",
        executor_session_id=f"executor-{i}",
        action_id=consent["action_id"],
        payload_digest=digest,
        protocol=ref("fixture.protocol", 4),
        permission_version=f"permission-{i}",
        revocation_version="2",
        scope=scope,
        budget=budget,
        issued_at_ns="10",
        expires_at_ns="15",
        tier=3,
    )
    if i % 3 == 1:
        grant["consent"] = consent
    command = dict(
        **version,
        command_id=f"command-{i}",
        kind=1 + (i % 3),
        law_version=ref("fixture.law", 5),
        history_prefix=f"prefix-{i}",
    )
    if command["kind"] == 3:
        command.update(
            verdict=1,
            alarm_record=ref("echo.system-maintenance-record", 9),
            alarm_id=f"alarm-{i}",
            review_record=ref("fixture.review", 6),
        )
    management: dict[str, Any] = dict(
        **version, command=command, user=user, entry=1 if i % 3 == 0 else 2
    )
    if management["entry"] == 1:
        management["confirmation_id"] = f"challenge-{i}"
    result = dict(**version, request_id=f"request-{i}", audit_record_id=f"audit-{i}")
    if i % 3 == 0:
        result["denied"] = 2
    elif i % 3 == 1:
        result["dispatch_record_id"] = f"dispatch-{i}"
    else:
        result["management_commit"] = sample("SystemMaintenanceRecord", i) | {"kind": 3}
    response = dict(
        **version,
        request_id=result["request_id"],
        session_id=user["session_id"],
        protocol_version=1,
        kind=4,
        reply_to=5,
    )
    if i % 3 != 1:
        response.update(reply_to=6, management=result)
        if "denied" in result:
            response.update(kind=5, reason=4)
    messages: dict[str, dict[str, Any]] = {
        "TerminalResponseV2": response,
        "LocalUserBinding": user,
        "DesktopPermissionPolicy": policy,
        "ActionConsent": consent,
        "ConnectorScope": scope,
        "ExecutionBudget": budget,
        "TrustMemory": trust,
        "ExecutionCredential": grant,
        "GrowthCommandV2": command,
        "GrowthManagementRequest": management,
        "PermissionResult": result,
        "TerminalRequestV2": dict(
            **version,
            request_id=f"request-{i}",
            session_id=user["session_id"],
            protocol_version=1,
            kind=6,
            management=management,
        ),
        "ActionRequestV2": dict(
            **version,
            action_id=grant["action_id"],
            expires_at_ns="15",
            grant=grant,
            action_payload=payload.hex(),
        ),
    }
    return messages[name]


def permission_legal() -> list[dict[str, Any]]:
    from fixture_inputs import request

    rows = []
    base = permission_sample("ExecutionCredential", 0)
    for kind in (1, 3, 4, 5, 6):
        scope: dict[str, Any] = dict(
            schema_version=1,
            kind=kind,
            connector_id="fixture",
            operation_id="declared-op",
            resource_id="owned-wallet" if kind >= 5 else "declared-target",
        )
        if kind >= 5:
            scope.update(wallet_minor_units="1", currency="CNY")
        if kind == 6:
            scope["depositor"] = permission_sample("LocalUserBinding", 0)
        grant = base | dict(scope=scope)
        if kind == 1:
            grant["trust"] = permission_sample("TrustMemory", 0) | dict(scope=scope)
        rows += [
            request("ConnectorScope", scope),
            request("ExecutionCredential", grant),
        ]
    review = permission_sample("GrowthCommandV2", 2)
    rows.append(
        request(
            "GrowthCommandV2",
            review | dict(verdict=2, replacement_law=ref("fixture.law", 8)),
        )
    )
    return rows


def permission_illegal() -> list[dict[str, Any]]:
    from fixture_inputs import request

    rows = []

    def mutate(name: str, path: str, value: Any, seed: int = 1) -> None:
        v = permission_sample(name, seed)
        obj = v
        keys = path.split(".")
        for field in keys[:-1]:
            obj = obj[field]
        if value is None:
            del obj[keys[-1]]
        else:
            obj[keys[-1]] = value
        rows.append(dict(label=f"permission-{name}-{path}", request=request(name, v)))

    for name, key, value in [
        ("ActionRequestV2", "grant", None),
        ("ActionRequestV2", "action_payload", "00"),
        ("ActionRequestV2", "action_id", "other"),
        ("ActionRequestV2", "expires_at_ns", "16"),
        ("ExecutionCredential", "executor_session_id", ""),
        ("ExecutionCredential", "issued_at_ns", "15"),
        ("ExecutionCredential", "scope.resource_id", ""),
        ("ExecutionCredential", "revocation_version", None),
        ("ExecutionCredential", "permission_version", ""),
        ("ExecutionCredential", "budget.charge", "5"),
        ("ExecutionCredential", "consent", None),
        ("ExecutionCredential", "consent.action_id", "other"),
        ("ExecutionCredential", "consent.payload_digest", "ee" * 32),
        ("ExecutionCredential", "consent.confirmed_at_ns", "11"),
        ("ExecutionCredential", "scope.desktop.irreversible", 1),
        ("ExecutionCredential", "scope.desktop.reversibility_rule", None),
        ("ExecutionCredential", "trust", permission_sample("TrustMemory", 0)),
        ("TrustMemory", "tier", 4),
        ("ConnectorScope", "kind", 5),
        ("TerminalRequestV2", "session_id", "wrong"),
        ("GrowthManagementRequest", "user", None),
        ("LocalUserBinding", "authentication_record_id", ""),
    ]:
        mutate(name, key, value)
    mutate("GrowthManagementRequest", "confirmation_id", None, 0)
    mutate(
        "GrowthManagementRequest", "command", permission_sample("GrowthCommandV2", 2), 0
    )
    mutate("TerminalResponseV2", "management.request_id", "other", 0)
    mutate("TerminalResponseV2", "management", None, 2)
    mutate("TerminalResponseV2", "reply_to", 1, 2)
    for key, value in [
        ("alarm_record.object_type", "wrong"),
        ("alarm_id", None),
        ("history_prefix", ""),
        ("law_version", None),
        ("review_record", None),
        ("verdict", 3),
        ("verdict", 2),
    ]:
        mutate("GrowthCommandV2", key, value, 2)
    mutate(
        "PermissionContext",
        "remote_text",
        permission_sample("PermissionContext", 0)["remote_text"],
    )
    from fixture_speak_think import receipt

    for key in (
        "action_choice",
        "query",
        "next_focus",
        "grant",
        "tool",
        "delivery",
        "write_h",
        "write_q",
    ):
        rows.append(
            dict(
                label=f"remote-forbidden-{key}",
                request=request("RemoteCognitiveReceipt", receipt(1) | {key: True}),
            )
        )
    return rows
