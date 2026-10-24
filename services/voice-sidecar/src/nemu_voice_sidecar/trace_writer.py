"""Append request traces without creating an unbounded number of files."""
from __future__ import annotations

import json
from pathlib import Path
from threading import Lock
from typing import Any

_WRITE_LOCK = Lock()


def append_jsonl(path: Path, record: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    line = json.dumps(record, separators=(",", ":")) + "\n"
    with _WRITE_LOCK:
        with path.open("a", encoding="utf-8", newline="\n") as stream:
            stream.write(line)
