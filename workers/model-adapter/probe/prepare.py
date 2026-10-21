"""Download pinned probe assets without importing torch or allocating GPU memory.

Metadata-only is the default. --weights explicitly includes safetensors. Each
successful download gets an inventory; no cached file is accepted as a result
of a failed download. This is preparation, never a carrier acceptance result.
"""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

HERE = Path(__file__).resolve().parent


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--weights", action="store_true")
    parser.add_argument("--model", help="Exact manifest repository id; default: all")
    args = parser.parse_args()
    manifest = json.loads((HERE / "manifest.json").read_text(encoding="utf-8"))
    models = manifest["models"]
    if args.model:
        models = [model for model in models if model["id"] == args.model]
        if not models:
            parser.error("model must be present in manifest.json")
    hf = Path(sys.executable).parent / ("hf.exe" if os.name == "nt" else "hf")
    if not hf.is_file():
        raise RuntimeError("Run with uv run --project workers/model-adapter/probe --locked")
    cache = HERE / ".cache"
    env = dict(os.environ, HF_HOME=str(cache / "huggingface"))
    for model in models:
        repo, revision = model["id"], model["revision"]
        if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repo):
            raise ValueError("invalid repository id")
        if not re.fullmatch(r"[0-9a-f]{40}", revision):
            raise ValueError("an immutable 40-character revision is required")
        target = cache / "models" / repo.replace("/", "--") / revision
        patterns = ["*.json", "*.txt", "*.model", "README.md", "LICENSE*"]
        if args.weights:
            patterns.append("*.safetensors")
        started = datetime.now(timezone.utc).isoformat()
        subprocess.run(
            [
                str(hf),
                "download",
                repo,
                "--revision",
                revision,
                "--local-dir",
                str(target),
                "--max-workers",
                "2",
                "--include",
                *patterns,
            ],
            env=env,
            check=True,
        )
        files = []
        for path in sorted(target.rglob("*")):
            relative = path.relative_to(target)
            if not path.is_file() or ".cache" in relative.parts:
                continue
            with path.open("rb") as stream:
                digest = hashlib.file_digest(stream, "sha256").hexdigest()
            files.append(dict(path=relative.as_posix(), bytes=path.stat().st_size, sha256=digest))
        result = dict(
            repository=repo,
            revision=revision,
            weights_requested=args.weights,
            started_at=started,
            completed_at=datetime.now(timezone.utc).isoformat(),
            files=files,
            status="downloaded-not-measured",
        )
        inventory = cache / "inventories"
        inventory.mkdir(parents=True, exist_ok=True)
        stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
        output = inventory / f"{repo.replace('/', '--')}-{stamp}.json"
        with output.open("x", encoding="utf-8") as stream:
            json.dump(result, stream, ensure_ascii=False, indent=2)
            stream.write("\n")
        print(output, flush=True)


if __name__ == "__main__":
    main()
