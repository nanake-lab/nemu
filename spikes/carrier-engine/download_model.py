"""P0-10 fixed-revision download using the existing restartable range downloader.

Only Python standard library and system curl are used. No model/GPU library is
loaded. Large files use 8 MiB ranges with four workers and bounded curl retries;
completed, range-validated chunks survive interruptions and process restarts.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath
import shutil
import subprocess
from typing import Callable, cast

ROOT = Path(__file__).resolve().parents[2]
REPOSITORY = "Qwen/Qwen3-4B-Instruct-2507"
REVISION = "cdbee75f17c01a7cc42f958dc650907174af0554"


def sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def git_sha1(path: Path) -> str:
    value = hashlib.sha1(f"blob {path.stat().st_size}\0".encode("ascii"))
    with path.open("rb") as stream:
        while data := stream.read(1024 * 1024):
            value.update(data)
    return value.hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--metadata", type=Path, required=True)
    args = parser.parse_args()
    curl = shutil.which("curl.exe") or shutil.which("curl")
    if curl is None:
        raise RuntimeError("System curl required")
    source = ROOT / "workers/model-adapter/probe/download_weights.py"
    spec = importlib.util.spec_from_file_location("p0_07_range_downloader", source)
    if spec is None or spec.loader is None:
        raise RuntimeError("Existing segmented downloader unavailable")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    download_chunks = cast(Callable[[str, str, Path, int], None], module.download_chunks)
    metadata = json.loads(args.metadata.read_text(encoding="utf-8-sig"))
    if metadata["sha"] != REVISION or metadata["id"] != REPOSITORY:
        raise ValueError("Fixed repository/revision mismatch")
    frozen_path = ROOT / "reports/P0-07/preparation/Qwen--Qwen3-4B-Instruct-2507-weights-20261016T151833171127Z.json"
    frozen = {item["path"]: item for item in json.loads(frozen_path.read_text(encoding="utf-8"))["files"]}
    target = args.cache_root.resolve() / "models" / REPOSITORY.replace("/", "--") / REVISION
    target.mkdir(parents=True, exist_ok=True)
    inventory = []
    started = datetime.now(timezone.utc).isoformat()
    for item in metadata["siblings"]:
        relative = PurePosixPath(item["rfilename"])
        if str(relative) == ".gitattributes":
            continue
        if relative.is_absolute() or ".." in relative.parts or "\\" in str(relative):
            raise ValueError("Unsafe model path")
        path = target.joinpath(*relative.parts)
        if not path.resolve().is_relative_to(target):
            raise ValueError("Path escaped model root")
        path.parent.mkdir(parents=True, exist_ok=True)
        size = int(item["size"])
        lfs = item.get("lfs")
        expected = lfs["sha256"] if lfs else item["blobId"]
        if str(relative) in frozen:
            known = frozen[str(relative)]
            if not lfs or size != known["bytes"] or expected != known["sha256"]:
                raise ValueError("Current weight metadata differs from frozen P0-07 hashes")
        partial = path.with_suffix(path.suffix + ".partial")
        if not path.exists():
            url = f"https://huggingface.co/{REPOSITORY}/resolve/{REVISION}/{relative}"
            print(f"Downloading {relative}: {size:,} bytes", flush=True)
            if lfs:
                download_chunks(curl, url, partial, size)
            else:
                subprocess.run([
                    curl, "--fail", "--location", "--silent", "--show-error",
                    "--connect-timeout", "20", "--max-time", "120",
                    "--retry", "7", "--retry-all-errors", "--output", str(partial), url,
                ], check=True)
        candidate = path if path.exists() else partial
        actual = sha256(candidate) if lfs else git_sha1(candidate)
        if candidate.stat().st_size != size or actual != expected:
            raise ValueError(f"Integrity mismatch: {relative}; file/chunks retained")
        if candidate == partial:
            partial.rename(path)
        inventory.append({"path": str(relative), "bytes": size, "sha256": sha256(path),
                          "upstreamDigest": expected, "upstreamDigestType": "sha256" if lfs else "git-sha1"})
        print(f"Verified {relative}: {inventory[-1]['sha256']}", flush=True)
    if not set(frozen).issubset({item["path"] for item in inventory}):
        raise ValueError("Not all frozen weight files downloaded")
    result = {"repository": REPOSITORY, "revision": REVISION, "target": str(target),
              "status": "downloaded-and-hash-verified-not-measured", "startedUtc": started,
              "endedUtc": datetime.now(timezone.utc).isoformat(), "files": inventory,
              "sourceSha256": sha256(Path(__file__)), "rangeDownloaderSha256": sha256(source),
              "metadataSha256": sha256(args.metadata)}
    inventories = args.cache_root.resolve() / "downloads" / "inventories"
    inventories.mkdir(parents=True, exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    output = inventories / f"qwen-{stamp}.json"
    with output.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, indent=2)
        stream.write("\n")
    print(f"Inventory: {output}", flush=True)


if __name__ == "__main__":
    main()
