from typing import Any
from echo import ingress as c, runtime as r
from echo.bridge import from_json


def check(q: dict[str, Any], limits: r.Limits) -> None:
    def field(key: str, name: str) -> dict[str, Any]:
        return r.normalize(name, from_json(name, q[key]), limits)

    match q["check"]:
        case "auth":
            c.auth_pair(
                field("request", "TerminalAuthenticate"),
                field("response", "TerminalAuthResult"),
                limits,
            )
        case "request":
            c.request(
                field("request", "TerminalRequest"),
                field("session", "TerminalSession") if q.get("session") else None,
                limits,
            )
        case "response":
            c.response(
                field("request", "TerminalRequest"),
                field("response", "TerminalResponse"),
                field("session", "TerminalSession"),
                limits,
            )
        case "window":
            c.window(
                field("chunk", "ObjectUploadChunk"),
                field("budget", "TerminalBudget"),
                q["next_offset"],
                q["active"],
                q["reserved"],
                q.get("declared_id", q["chunk"]["upload_id"]),
                from_json("ObjectRef", q.get("declared_object", q["chunk"]["object"])),
                limits,
            )
        case "chunk":
            c.chunk(
                field("chunk", "ObjectUploadChunk"),
                field("budget", "TerminalBudget"),
                limits,
            )
        case "object":
            c.uploaded(
                field("reference", "ObjectRef"),
                bytes.fromhex(q["hex"]),
                field("budget", "TerminalBudget"),
                limits,
            )
        case _:
            raise ValueError("unknown ingress check")
