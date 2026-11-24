"""ADR 0051 synthetic calls. The grant, sent bytes and received prefix are independent."""

from __future__ import annotations

import hashlib
from typing import Any


def sample_irl(name: str, seed: int) -> dict[str, Any]:
    from fixture_inputs import SCHEMA

    def object_ref(data: bytes) -> dict[str, Any]:
        return dict(
            schema_version=1,
            object_type="nemu.storage.raw-bytes-v1",
            resource_version="1",
            digest=hashlib.sha256(data).hexdigest(),
            byte_length=str(len(data)),
        )

    version = dict(schema_version=1)
    call = version | dict(
        protocol_version=1, session_id=f"pipe-{seed}", request_id=f"request-{seed}"
    )
    data = f"本机合成输入 {seed}".encode()
    ref = object_ref(data)
    material = version | dict(object=ref, data=data.hex())
    borrow = version | dict(grant_id=f"grant-{seed}", object=ref)
    dispatch = version | dict(
        target=version | dict(branch_id="fixture", record_id="event", content=ref),
        payload=borrow,
        facts=version | dict(grant_id="facts", object=object_ref(b"facts")),
        registry=version | dict(grant_id="registry", object=object_ref(b"registry")),
    )
    delegation: dict[str, Any] = version | dict(
        link=2 if seed % 2 == 0 else 3,
        parent=call | dict(session_id="parent"),
        inputs=[material],
        parameters=object_ref(b"decode"),
    )
    if delegation["link"] == 2:
        delegation.update(
            provider="fixture",
            requested_model="example",
            disclosure=object_ref(b"permission"),
        )
    else:
        delegation.update(
            local_model=object_ref(b"model"), isolation=object_ref(b"neutral")
        )
    operation = 1 + seed % (len(SCHEMA["enums"]["IrlPipeOperation"]) - 1)
    reason = version | dict(code=3)
    fields: list[dict[str, Any]] = [
        {"dispatch": dispatch},
        {"delegation": delegation},
        {"borrow": borrow},
        dict(borrow=borrow, material=material),
        {},
        {},
        dict(reason=reason, remote_status=6),
        dict(reason=reason, sent_request=object_ref(b"sent")),
        dict(reason=reason),
        dict(reason=reason),
        dict(graph=object_ref(b"graph"), output=material),
        dict(output=material),
        dict(output=material),
    ]
    samples: dict[str, dict[str, Any]] = {
        "IrlPipeAuth": version
        | dict(
            protocol_version=1,
            link=1 + seed % 3,
            session_id=call["session_id"],
            launch_credential=bytes([1 + seed] * 32).hex(),
        ),
        "IrlCall": call,
        "IrlPipeBudget": version
        | dict(
            version=f"limits-{seed}",
            frame_bytes=8192,
            object_bytes="4096",
            output_bytes="2048",
        ),
        "IrlBorrow": borrow,
        "IrlMaterial": material,
        "IrlDispatch": dispatch,
        "IrlDelegation": delegation,
        "IrlPipeFrame": version
        | dict(call=call, operation=operation)
        | fields[operation - 1],
    }
    return samples[name]


def illegal_irl() -> list[dict[str, Any]]:
    from fixture_inputs import request

    def bad(label: str, seed: int, **changes: Any) -> dict[str, Any]:
        return dict(
            label=label,
            request=request("IrlPipeFrame", sample_irl("IrlPipeFrame", seed) | changes),
        )

    return [
        bad(
            "not-sent-with-receipt",
            6,
            sent_request=sample_irl("IrlBorrow", 0)["object"],
        ),
        bad("not-sent-with-completed-status", 6, remote_status=1),
        bad("unknown-with-invented-output", 7, output=sample_irl("IrlMaterial", 0)),
        bad("cancel-request-with-output", 4, output=sample_irl("IrlMaterial", 0)),
    ]
