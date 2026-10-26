"""Download pinned official Windows CUDA archives without installing or running them."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
from typing import Callable, cast

ROOT = Path(__file__).resolve().parents[2]
TAG = "b11669"
COMMIT = "620d2225cc436ab0b001e3200e590e3b286ef9ab"
ASSETS = {
    "llama-b11669-bin-win-cuda-12.4-x64.zip":
        "c2da1f73ed176ae093ac5aceb978a7b167a16113f6d74c6789eeaee2135cb8dc",
    "cudart-llama-bin-win-cuda-12.4-x64.zip":
        "8c79a9b226de4b3cacfd1f83d24f962d0773be79f1e7b75c6af4ded7e32ae1d6",
}


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


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
    if metadata["tag_name"] != TAG or metadata["target_commitish"] != COMMIT:
        raise ValueError("Pinned official release differs")
    available = {asset["name"]: asset for asset in metadata["assets"]}
    target = args.cache_root.resolve() / "downloads" / "engine" / TAG
    target.mkdir(parents=True, exist_ok=True)
    inventory = []
    for name, expected in ASSETS.items():
        asset = available[name]
        if asset["digest"] != "sha256:" + expected:
            raise ValueError("Release asset digest mismatch")
        url = f"https://github.com/ggml-org/llama.cpp/releases/download/{TAG}/{name}"
        if asset["browser_download_url"] != url:
            raise ValueError("Unexpected asset URL")
        size = int(asset["size"])
        path = target / name
        partial = path.with_suffix(".zip.partial")
        if not path.exists():
            print(f"Downloading {name}: {size:,} bytes", flush=True)
            download_chunks(curl, url, partial, size)
        candidate = path if path.exists() else partial
        actual = digest(candidate)
        if candidate.stat().st_size != size or actual != expected:
            raise ValueError(f"Integrity mismatch: {name}; all chunks retained")
        if candidate == partial:
            partial.rename(path)
        inventory.append({"path": name, "bytes": size, "sha256": actual, "source": url})
        print(f"Verified {name}: {actual}", flush=True)
    result = {"repository": "ggml-org/llama.cpp", "tag": TAG, "commit": COMMIT,
              "target": str(target), "status": "archives-verified-not-extracted-or-run",
              "files": inventory, "sourceSha256": digest(Path(__file__)),
              "rangeDownloaderSha256": digest(source), "metadataSha256": digest(args.metadata),
              "endedUtc": datetime.now(timezone.utc).isoformat()}
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    output = target / f"inventory-{stamp}.json"
    with output.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, indent=2)
        stream.write("\n")
    print(f"Inventory: {output}", flush=True)


if __name__ == "__main__":
    main()
