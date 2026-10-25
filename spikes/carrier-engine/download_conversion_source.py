"""Retrieve the pinned converter source tree, verifying each Git blob before use."""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import subprocess

COMMIT = "620d2225cc436ab0b001e3200e590e3b286ef9ab"
EXACT = {
    "convert_hf_to_gguf.py", "LICENSE", "gguf-py/LICENSE", "gguf-py/pyproject.toml",
    "gguf-py/README.md", "requirements/requirements-convert_hf_to_gguf.txt",
    "requirements/requirements-convert_legacy_llama.txt",
    "tools/quantize/quantize.cpp", "tools/quantize/main.cpp",
    "tools/quantize/README.md", "ggml/src/ggml-backend-reg.cpp", "common/common.cpp",
}


def sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def git_blob(path: Path) -> str:
    value = hashlib.sha1(f"blob {path.stat().st_size}\0".encode("ascii"))
    with path.open("rb") as stream:
        while data := stream.read(1024 * 1024):
            value.update(data)
    return value.hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tree", type=Path, required=True)
    parser.add_argument("--target", type=Path, required=True)
    args = parser.parse_args()
    curl = shutil.which("curl.exe") or shutil.which("curl")
    if curl is None:
        raise RuntimeError("System curl required")
    tree = json.loads(args.tree.read_text(encoding="utf-8-sig"))
    if tree["sha"] != COMMIT or tree["truncated"]:
        raise ValueError("Pinned source tree mismatch or truncated inventory")
    entries = [entry for entry in tree["tree"] if entry["type"] == "blob"
               and (entry["path"] in EXACT or entry["path"].startswith(
                   ("conversion/", "gguf-py/gguf/", "include/", "ggml/include/")))]
    if not EXACT.issubset({entry["path"] for entry in entries}):
        raise ValueError("Required source files missing")
    target = args.target.resolve()
    target.mkdir(parents=True, exist_ok=True)

    def fetch(entry: dict[str, str | int]) -> dict[str, str | int]:
        relative = PurePosixPath(str(entry["path"]))
        if relative.is_absolute() or ".." in relative.parts or "\\" in str(relative):
            raise ValueError("Unsafe source path")
        path = target.joinpath(*relative.parts)
        if not path.resolve().is_relative_to(target):
            raise ValueError("Source path escaped target")
        path.parent.mkdir(parents=True, exist_ok=True)
        partial = path.with_suffix(path.suffix + ".partial")
        url = f"https://raw.githubusercontent.com/ggml-org/llama.cpp/{COMMIT}/{relative}"
        if not path.exists():
            subprocess.run([
                curl, "--fail", "--location", "--silent", "--show-error",
                "--connect-timeout", "20", "--max-time", "120", "--retry", "3",
                "--retry-all-errors", "--output", str(partial), url,
            ], check=True)
        candidate = path if path.exists() else partial
        if candidate.stat().st_size != int(entry["size"]) or git_blob(candidate) != entry["sha"]:
            raise ValueError(f"Source integrity mismatch: {relative}; file retained")
        if candidate == partial:
            partial.rename(path)
        return {"path": str(relative), "bytes": path.stat().st_size,
                "gitBlob": str(entry["sha"]), "sha256": sha256(path), "source": url}

    files = []
    with ThreadPoolExecutor(max_workers=2) as pool:
        for index, result in enumerate(pool.map(fetch, entries), 1):
            files.append(result)
            if index % 25 == 0 or index == len(entries):
                print(f"Verified source files: {index}/{len(entries)}", flush=True)
    record = {"commit": COMMIT, "status": "source-hash-verified-not-run",
              "target": str(target), "files": files, "treeSha256": sha256(args.tree),
              "sourceSha256": sha256(Path(__file__)),
              "endedUtc": datetime.now(timezone.utc).isoformat()}
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    output = target / f"source-inventory-{stamp}.json"
    with output.open("x", encoding="utf-8") as stream:
        json.dump(record, stream, indent=2)
        stream.write("\n")
    print(f"Inventory: {output}", flush=True)


if __name__ == "__main__":
    main()
