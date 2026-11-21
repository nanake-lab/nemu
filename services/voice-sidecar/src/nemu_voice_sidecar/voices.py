"""Only registered, hash-bound synthetic or explicitly licensed references are usable."""
from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Reference:
    path: Path
    text: str
    sha256: str


class VoiceBank:
    def __init__(self, path: Path) -> None:
        self.path = path

    def select(self, voice: str, lang: str, style: str) -> Reference:
        data = json.loads(self.path.read_text(encoding="utf-8-sig"))
        if data["id"] != voice:
            raise ValueError("VOICE_NOT_REGISTERED")
        ref = data["references"][lang][style]
        # Never accept arbitrary audio paths from a synthesis request.
        if ref["provenance"] not in ("VoiceDesign", "licensed-dataset"):
            raise ValueError("VOICE_PROVENANCE_REJECTED")
        if not ref["source"] or not ref["license"] or ref["cloneAllowed"] is not True:
            raise ValueError("VOICE_LICENSE_MISSING")
        root = self.path.parent.resolve()
        target = (root / ref["path"]).resolve()
        if not target.is_relative_to(root):
            raise ValueError("VOICE_PATH_REJECTED")
        with target.open("rb") as stream:
            digest = hashlib.file_digest(stream, "sha256").hexdigest()
        if digest != ref["sha256"]:
            raise ValueError("VOICE_HASH_MISMATCH")
        return Reference(target, str(ref["text"]), digest)
