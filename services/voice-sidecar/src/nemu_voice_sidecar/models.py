"""Pinned offline model inventory. Startup never downloads or imports CUDA libraries."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[4]


def verified_model(cache: Path, variant: str, lock_path: Path = ROOT / "config/models.lock.json") -> Path:
    lock = json.loads(lock_path.read_text(encoding="utf-8-sig"))
    model = lock["tts"]["models"][variant]
    folder = cache / str(model["id"]).replace("/", "--") / str(model["revision"])
    for item in model["files"]:
        relative = PurePosixPath(item["path"])
        if relative.is_absolute() or ".." in relative.parts:
            raise ValueError("MODEL_PATH_REJECTED")
        target = folder.joinpath(*relative.parts)
        if not target.is_file() or target.stat().st_size != item["size"]:
            raise ValueError("MODEL_FILES_MISSING")
        with target.open("rb") as stream:
            digest = hashlib.file_digest(stream, "sha256").hexdigest()
        if digest != item["sha256"]:
            raise ValueError("MODEL_HASH_MISMATCH")
    return folder
