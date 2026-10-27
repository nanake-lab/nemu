"""ADR 0024 pure checks. The session argument comes from an authenticated connection.
Decoding a session never authenticates it. No token, service or persistent state here.
"""

from __future__ import annotations
from typing import Any
from . import runtime as r

UPLOAD_TYPES = {
    m["tag"]: name
    for name, m in r.SCHEMA["messages"].items()
    if not m.get("transient")
    and not m.get("superseded")
    and m["file"] != "terminal_ingress.proto"
}


def window(
    c: dict[str, Any],
    b: dict[str, Any],
    next_offset: int,
    active: int,
    reserved: int,
    declared_id: str,
    declared_object: dict[str, Any],
    limits: r.Limits,
) -> None:
    chunk(c, b, limits)
    r.fail(
        c["upload_id"] == declared_id and c["object"] == declared_object,
        "upload declaration changed",
    )
    r.fail(c["offset"] == next_offset, "nonsequential upload")
    r.fail(
        0 < active <= b["max_uploads"]
        and c["object"]["byte_length"] <= reserved <= b["max_upload_bytes"],
        "session upload budget",
    )


def session(s: dict[str, Any], limits: r.Limits) -> None:
    r.normalize("TerminalSession", s, limits)
    b = s["budget"]
    r.fail(
        b["max_chunk_bytes"] <= b["max_object_bytes"] <= b["max_upload_bytes"]
        and b["max_chunk_bytes"] < b["max_frame_bytes"],
        "terminal budget",
    )


def auth_pair(q: dict[str, Any], a: dict[str, Any], limits: r.Limits) -> None:
    r.normalize("TerminalAuthenticate", q, limits)
    r.normalize("TerminalAuthResult", a, limits)
    r.fail(q["request_id"] == a["request_id"], "auth pairing")
    if "session" in a:
        s = a["session"]
        session(s, limits)
        r.fail(
            q["installation_id"] == s["installation_id"]
            and s["protocol_version"] in q["protocol_versions"],
            "auth negotiation",
        )


def chunk(c: dict[str, Any], b: dict[str, Any], limits: r.Limits) -> None:
    r.normalize("ObjectUploadChunk", c, limits)
    r.normalize("TerminalBudget", b, limits)
    n = len(c["data"])
    end = c["offset"] + n
    r.fail(
        0 < n <= b["max_chunk_bytes"]
        and c["object"]["byte_length"] <= b["max_object_bytes"],
        "chunk budget",
    )
    r.fail(
        end < 2**64
        and end <= c["object"]["byte_length"]
        and c["final_chunk"] == (end == c["object"]["byte_length"]),
        "chunk extent",
    )
    r.fail(c["object"]["object_type"] in UPLOAD_TYPES, "upload type")


def uploaded(
    ref: dict[str, Any], data: bytes, b: dict[str, Any], limits: r.Limits
) -> None:
    r.normalize("ObjectRef", ref, limits)
    r.normalize("TerminalBudget", b, limits)
    r.fail(
        len(data) == ref["byte_length"] and len(data) <= b["max_object_bytes"],
        "object size",
    )
    name = UPLOAD_TYPES[ref["object_type"]]
    v = r.decode_canonical(name, data, limits)
    r.fail(r.digest(name, v, limits) == ref["digest"].hex(), "object digest")


def request(q: dict[str, Any], s: dict[str, Any] | None, limits: r.Limits) -> None:
    data = r.encode_protobuf("TerminalRequest", q, limits)
    r.fail(s is not None, "unauthenticated dispatch")
    assert s is not None
    session(s, limits)
    r.fail(
        q["session_id"] == s["session_id"]
        and q["protocol_version"] == s["protocol_version"],
        "session mismatch",
    )
    r.fail(
        q["kind"] in s["capabilities"] and len(data) <= s["budget"]["max_frame_bytes"],
        "request budget or capability",
    )
    if "event" in q:
        r.fail(
            q["event"]["payload_digest"] == q["event"]["payload_ref"]["digest"],
            "event digest",
        )
    if "upload" in q:
        chunk(q["upload"], s["budget"], limits)


def response(
    q: dict[str, Any], a: dict[str, Any], s: dict[str, Any], limits: r.Limits
) -> None:
    request(q, s, limits)
    r.fail(
        len(r.encode_protobuf("TerminalResponse", a, limits))
        <= s["budget"]["max_frame_bytes"],
        "response budget",
    )
    r.fail(
        all(q[k] == a[k] for k in ["request_id", "session_id", "protocol_version"])
        and q["kind"] == a["reply_to"],
        "response pairing",
    )
    if a["kind"] == 5:
        return
    expected = q["kind"] if q["kind"] <= 3 else 4
    r.fail(a["kind"] == expected, "reply kind")
    if expected == 1:
        r.fail(
            q["event"]["id"] == a["receipt"]["event_id"]
            and q["event"]["stable_source_key"] == a["receipt"]["stable_source_key"],
            "event pairing",
        )
    elif expected == 2:
        c, v = q["upload"], a["upload"]
        r.fail(
            c["upload_id"] == v["upload_id"]
            and c["object"] == v["object"]
            and c["offset"] + len(c["data"]) == v["next_offset"]
            and (v["status"] == 2) == c["final_chunk"],
            "upload pairing",
        )
    elif expected == 3:
        r.fail(q["query"]["query_id"] == a["query"]["query_id"], "query pairing")
