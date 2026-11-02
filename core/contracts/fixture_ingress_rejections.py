"""PR #110 R1 oracles: rejection pairing is independent of request admission."""

from copy import deepcopy
from typing import Any


def vectors(existing: list[dict[str, Any]]) -> list[dict[str, Any]]:
    seed = deepcopy(
        next(v["request"] for v in existing if v["label"] == "durable-receipt-pair")
    )
    q, a, s = seed["request"], seed["response"], seed["session"]
    rejection = {k: v for k, v in a.items() if k != "receipt"} | dict(kind=5, reason=4)
    out: list[dict[str, Any]] = []

    def add(label: str, ok: bool, **fields: Any) -> None:
        out.append(
            dict(
                label="r1-" + label,
                request=deepcopy(seed | fields),
                expected=dict(ok=ok),
            )
        )

    bad_digest = deepcopy(q)
    bad_digest["event"]["payload_digest"] = "00" * 32
    short_frame = s | dict(
        budget=s["budget"] | dict(max_frame_bytes=128, max_chunk_bytes=1)
    )
    chunk = deepcopy(
        next(
            v["request"]["chunk"]
            for v in existing
            if v["label"] == "upload-complete-chunk"
        )
    )
    upload_q = {k: v for k, v in q.items() if k != "event"} | dict(kind=2, upload=chunk)
    upload_a = {k: v for k, v in a.items() if k != "receipt"} | dict(
        kind=2,
        reply_to=2,
        upload=dict(
            schema_version=1,
            upload_id=chunk["upload_id"],
            object=chunk["object"],
            next_offset=str(len(bytes.fromhex(chunk["data"]))),
            status=2,
        ),
    )
    # The request must fail admission; its paired rejection must pass, and a
    # success response to that same request/session must still fail.
    for label, request, success, session, reason in [
        ("capability", q, a, s | dict(capabilities=[3]), 4),
        ("event-digest", bad_digest, a, s, 6),
        ("request-frame-budget", q, a, short_frame, 1),
        (
            "upload-budget",
            upload_q,
            upload_a,
            s | dict(budget=s["budget"] | dict(max_chunk_bytes=1)),
            10,
        ),
    ]:
        rejected = rejection | dict(reply_to=request["kind"], reason=reason)
        add(
            label + "-admission",
            False,
            check="request",
            request=request,
            session=session,
        )
        add(
            label + "-rejection",
            True,
            request=request,
            response=rejected,
            session=session,
        )
        add(
            label + "-success-denied",
            False,
            request=request,
            response=success,
            session=session,
        )

    limited = s | dict(capabilities=[3])
    mutations: list[tuple[str, dict[str, Any]]] = [
        ("wrong-id", dict(request_id="other")),
        ("wrong-session", dict(session_id="other")),
        ("wrong-version", dict(protocol_version=2)),
        ("wrong-kind", dict(reply_to=2)),
        ("unspecified-reason", dict(reason=0)),
        ("unknown-reason", dict(reason=99)),
        ("mixed-success-payload", dict(receipt=a["receipt"])),
    ]
    for label, changed in mutations:
        add("rejection-" + label, False, response=rejection | changed, session=limited)
    add(
        "rejection-missing-reason",
        False,
        response={k: v for k, v in rejection.items() if k != "reason"},
        session=limited,
    )
    add(
        "rejection-wrong-connection",
        False,
        request=q | dict(session_id="other"),
        response=rejection | dict(session_id="other"),
        session=limited,
    )
    add(
        "rejection-malformed-request",
        False,
        request={k: v for k, v in q.items() if k != "event"},
        response=rejection,
        session=limited,
    )
    add(
        "rejection-unknown-request-field",
        False,
        request=q | dict(raw_log_position="0"),
        response=rejection,
        session=limited,
    )
    add(
        "rejection-response-frame-budget",
        False,
        response=rejection,
        session=s
        | dict(budget=s["budget"] | dict(max_frame_bytes=32, max_chunk_bytes=1)),
    )
    add(
        "rejection-inconsistent-budget",
        False,
        response=rejection,
        session=s | dict(budget=s["budget"] | dict(max_chunk_bytes=8192)),
    )
    return out
