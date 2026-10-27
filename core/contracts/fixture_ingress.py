"""Explicit contextual oracles, frozen independently from codec vectors.
Expected decisions are specified here, not calculated by the implementation.
"""

from typing import Any
from copy import deepcopy
import json
from pathlib import Path


def vectors() -> list[dict[str, Any]]:
    from fixture_inputs import LIMITS, sample

    old = json.loads(
        (Path(__file__).parent / "golden/legal.json").read_text(encoding="utf-8")
    )
    payload = next(v["expected"] for v in old if v["request"]["name"] == "UserText")
    ref: dict[str, Any] = dict(
        schema_version=1,
        digest=payload["digest"],
        object_type="echo.user-text",
        resource_version="1",
        byte_length=str(len(bytes.fromhex(payload["canonical"]))),
    )
    budget: dict[str, Any] = dict(
        schema_version=1,
        version="test-limits-1",
        max_frame_bytes=8192,
        max_chunk_bytes=1024,
        max_object_bytes=4096,
        max_uploads=2,
        max_upload_bytes=8192,
    )
    session: dict[str, Any] = dict(
        schema_version=1,
        session_id="server-session-A",
        installation_id="installation-A",
        protocol_version=1,
        capabilities=[1, 2, 3, 4, 5, 6],
        budget=budget,
    )
    event = sample("TerminalEvent", 0) | dict(
        id="event-1",
        stable_source_key="stable-1",
        payload_ref=ref,
        provenance=ref,
        payload_digest=ref["digest"],
    )
    request: dict[str, Any] = dict(
        schema_version=1,
        request_id="request-1",
        session_id=session["session_id"],
        protocol_version=1,
        kind=1,
        event=event,
    )
    receipt: dict[str, Any] = dict(
        schema_version=1,
        stable_source_key=event["stable_source_key"],
        event_id=event["id"],
        stage=1,
        raw_log_position="0",
        ingest_time_ns="123",
    )
    response: dict[str, Any] = dict(
        schema_version=1,
        request_id=request["request_id"],
        session_id=session["session_id"],
        protocol_version=1,
        reply_to=1,
        kind=1,
        receipt=receipt,
    )
    chunk: dict[str, Any] = dict(
        schema_version=1,
        upload_id="upload-1",
        object=ref,
        offset="0",
        data=payload["canonical"],
        final_chunk=True,
    )
    auth: dict[str, Any] = dict(
        schema_version=1,
        request_id="auth-1",
        installation_id=session["installation_id"],
        token="ab" * 32,
        protocol_versions=[1],
    )
    answer: dict[str, Any] = dict(
        schema_version=1, request_id=auth["request_id"], status=1, session=session
    )
    out: list[dict[str, Any]] = []

    def add(label: str, ok: bool, check: str, **fields: Any) -> None:
        out.append(
            dict(
                label=label,
                request=deepcopy(
                    dict(
                        name="TerminalRequest",
                        op="ingress",
                        check=check,
                        limits=LIMITS,
                        **fields,
                    )
                ),
                expected=dict(ok=ok),
            )
        )

    add("first-authentication", True, "auth", request=auth, response=answer)
    reconnect = deepcopy(answer)
    reconnect["session"]["session_id"] = "server-session-B"
    add("reconnect-new-session", True, "auth", request=auth, response=reconnect)
    add(
        "authentication-failure",
        True,
        "auth",
        request=auth,
        response=dict(
            schema_version=1, request_id=auth["request_id"], status=2, reason=2
        ),
    )
    mismatch = deepcopy(answer)
    mismatch["request_id"] = "other"
    add("wrong-auth-request-id", False, "auth", request=auth, response=mismatch)
    add("event-submit", True, "request", request=request, session=session)
    add("unauthenticated-dispatch", False, "request", request=request, session=None)
    add(
        "stale-session-after-reconnect",
        False,
        "request",
        request=request,
        session=reconnect["session"],
    )
    limited = deepcopy(session)
    limited["capabilities"] = [3]
    add("capability-denied", False, "request", request=request, session=limited)
    add(
        "unknown-dispatch",
        False,
        "request",
        request=request | dict(kind=99),
        session=session,
    )
    forged = deepcopy(request)
    forged["event"]["raw_log_position"] = "3"
    add("client-forged-position", False, "request", request=forged, session=session)
    forged = deepcopy(request)
    forged["event"]["ingest_time_ns"] = "3"
    add("client-forged-time", False, "request", request=forged, session=session)
    add(
        "durable-receipt-pair",
        True,
        "response",
        request=request,
        response=response,
        session=session,
    )
    retry = deepcopy(request)
    retry["request_id"] = "retry"
    retry["session_id"] = reconnect["session"]["session_id"]
    retry_reply = deepcopy(response)
    retry_reply.update(request_id="retry", session_id=retry["session_id"])
    retry_reply["receipt"]["raw_log_position"] = "1"
    add(
        "retry-same-source-new-position",
        True,
        "response",
        request=retry,
        response=retry_reply,
        session=reconnect["session"],
    )
    wrong = deepcopy(response)
    wrong["receipt"]["stable_source_key"] = "new-key"
    add(
        "wrong-source-pair",
        False,
        "response",
        request=request,
        response=wrong,
        session=session,
    )
    wrong = deepcopy(response)
    wrong["receipt"]["reason"] = "legacy-string"
    add(
        "mixed-old-new-receipt",
        False,
        "response",
        request=request,
        response=wrong,
        session=session,
    )
    add(
        "typed-backpressure",
        True,
        "response",
        request=request,
        response={
            k: v
            for k, v in (response | dict(kind=5, reason=7)).items()
            if k != "receipt"
        },
        session=session,
    )
    add("upload-complete-chunk", True, "chunk", chunk=chunk, budget=budget)
    first = deepcopy(chunk)
    first["data"] = first["data"][:2]
    first["final_chunk"] = False
    add("upload-disconnect-no-completion", True, "chunk", chunk=first, budget=budget)
    add(
        "oversized-chunk",
        False,
        "chunk",
        chunk=chunk,
        budget=budget | dict(max_chunk_bytes=1),
    )
    add(
        "premature-final",
        False,
        "chunk",
        chunk=first | dict(final_chunk=True),
        budget=budget,
    )
    add(
        "overflow-offset",
        False,
        "chunk",
        chunk=chunk | dict(offset=str(2**64 - 1)),
        budget=budget,
    )
    add(
        "canonical-object-complete",
        True,
        "object",
        reference=ref,
        hex=payload["canonical"],
        budget=budget,
    )
    add(
        "object-digest-mismatch",
        False,
        "object",
        reference=ref | dict(digest="00" * 32),
        hex=payload["canonical"],
        budget=budget,
    )
    add(
        "credential-not-uploadable",
        False,
        "object",
        reference=ref | dict(object_type="echo.terminal-authenticate"),
        hex=payload["canonical"],
        budget=budget,
    )
    for kind, field, name in [
        (3, "query", "QueryV2"),
        (4, "approval", "ApprovalDecisionV2"),
        (5, "cancel", "OutputCancel"),
        (6, "growth", "GrowthControl"),
    ]:
        q = {k: v for k, v in request.items() if k != "event"}
        q.update(kind=kind, **{field: sample(name, 0)})
        a = {k: v for k, v in response.items() if k != "receipt"}
        a.update(reply_to=kind, kind=3 if kind == 3 else 4)
        if kind == 3:
            a["query"] = sample("QueryResultV2", 0) | dict(
                query_id=q["query"]["query_id"]
            )
        add("paired-" + field, True, "response", request=q, response=a, session=session)
        add(
            "wrong-dispatch-pair-" + field,
            False,
            "response",
            request=q,
            response=a | dict(reply_to=1),
            session=session,
        )
    add(
        "upload-window-valid",
        True,
        "window",
        chunk=chunk,
        budget=budget,
        next_offset=0,
        active=1,
        reserved=int(ref["byte_length"]),
    )
    add(
        "upload-nonsequential",
        False,
        "window",
        chunk=chunk,
        budget=budget,
        next_offset=1,
        active=1,
        reserved=int(ref["byte_length"]),
    )
    add(
        "upload-count-budget",
        False,
        "window",
        chunk=chunk,
        budget=budget,
        next_offset=0,
        active=3,
        reserved=int(ref["byte_length"]),
    )
    add(
        "upload-total-byte-budget",
        False,
        "window",
        chunk=chunk,
        budget=budget,
        next_offset=0,
        active=1,
        reserved=8193,
    )
    add(
        "upload-declaration-id-changed",
        False,
        "window",
        chunk=chunk,
        budget=budget,
        next_offset=0,
        active=1,
        reserved=int(ref["byte_length"]),
        declared_id="other",
    )
    add(
        "upload-declaration-object-changed",
        False,
        "window",
        chunk=chunk,
        budget=budget,
        next_offset=0,
        active=1,
        reserved=int(ref["byte_length"]),
        declared_object=ref | dict(resource_version="other"),
    )
    return out


if __name__ == "__main__":
    target = Path(__file__).parent / "golden/ingress.json"
    assert not target.exists(), "frozen contextual oracles must not be overwritten"
    target.write_text(
        json.dumps(vectors(), indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
