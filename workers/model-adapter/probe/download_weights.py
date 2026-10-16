"""System-curl fallback: immutable Hugging Face revision, verified LFS SHA-256.

Does not disable TLS verification. HTTP(S)_PROXY can be set for this process.
Partial files stay in ignored cache; rename to a model weight only after the
size and digest both match the repository's pinned metadata.
"""

from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import hashlib
import json
import re
from pathlib import Path, PurePosixPath
import shutil
import subprocess

HERE = Path(__file__).resolve().parent


def download_chunks(curl: str, url: str, partial: Path, size: int) -> None:
    """Bounded range requests avoid stalled multi-GB responses; resume verified ranges."""
    chunk_size = 8 * 1024 * 1024
    folder = partial.with_suffix(".chunks")
    folder.mkdir(exist_ok=True)
    ranges = [(start, min(start + chunk_size, size) - 1) for start in range(0, size, chunk_size)]

    def fetch(bounds: tuple[int, int]) -> Path:
        start, end = bounds
        chunk = folder / f"{start:012d}.bin"
        header = folder / f"{start:012d}.headers"

        def valid() -> bool:
            if not chunk.exists() or not header.exists() or chunk.stat().st_size != end - start + 1:
                return False
            headers = header.read_text(encoding="utf-8", errors="replace")
            matches = re.findall(r"(?im)^content-range:\s*bytes (\d+)-(\d+)/(\d+)\s*$", headers)
            return bool(matches) and tuple(map(int, matches[-1])) == (start, end, size)

        if not valid():
            subprocess.run(
                [
                    curl,
                    "--fail",
                    "--location",
                    "--silent",
                    "--show-error",
                    "--connect-timeout",
                    "30",
                    "--max-time",
                    "120",
                    "--retry",
                    "3",
                    "--retry-all-errors",
                    "--range",
                    f"{start}-{end}",
                    "--max-filesize",
                    str(end - start + 1),
                    "--dump-header",
                    str(header),
                    "--output",
                    str(chunk),
                    url,
                ],
                check=True,
            )
        if not valid():
            raise ValueError(f"range/length mismatch at {start}; do not assemble")
        return chunk

    with ThreadPoolExecutor(max_workers=4) as pool:
        for i, _ in enumerate(pool.map(fetch, ranges), 1):
            if i % 16 == 0 or i == len(ranges):
                print(f"Ranges verified: {i}/{len(ranges)}", flush=True)
    with partial.open("wb") as stream:
        for start, _ in ranges:
            with (folder / f"{start:012d}.bin").open("rb") as chunk:
                shutil.copyfileobj(chunk, stream)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model", help="Exact manifest repository id; default: all")
    args = parser.parse_args()
    curl = shutil.which("curl.exe") or shutil.which("curl")
    if curl is None:
        raise RuntimeError("system curl required")
    manifest = json.loads((HERE / "manifest.json").read_text(encoding="utf-8"))
    models = [m for m in manifest["models"] if args.model is None or args.model == m["id"]]
    if not models:
        parser.error("unknown model")
    for model in models:
        repo, revision = model["id"], model["revision"]
        result = subprocess.run(
            [
                curl,
                "--fail",
                "--silent",
                "--show-error",
                "--location",
                "--max-time",
                "60",
                f"https://huggingface.co/api/models/{repo}/revision/{revision}?blobs=true",
            ],
            check=True,
            capture_output=True,
            encoding="utf-8",
        )
        metadata = json.loads(result.stdout)
        if metadata["sha"] != revision:
            raise ValueError("API revision mismatch")
        target = HERE / ".cache/models" / repo.replace("/", "--") / revision
        target.mkdir(parents=True, exist_ok=True)
        (target / ".cache").mkdir(exist_ok=True)
        (target / ".cache/weight-metadata.json").write_text(result.stdout, encoding="utf-8")
        inventory = []
        for item in metadata["siblings"]:
            relative = PurePosixPath(item["rfilename"])
            if relative.suffix != ".safetensors":
                continue
            if relative.is_absolute() or ".." in relative.parts:
                raise ValueError("invalid repository path")
            size, expected = item["lfs"]["size"], item["lfs"]["sha256"]
            path = target.joinpath(*relative.parts)
            if not path.resolve().is_relative_to(target.resolve()):
                raise ValueError("weight path escaped the model cache")
            path.parent.mkdir(parents=True, exist_ok=True)
            partial = path.with_suffix(path.suffix + ".partial")
            if not path.exists():
                print(f"Downloading {repo}@{revision}/{relative} ({size} bytes)", flush=True)
                download_chunks(
                    curl,
                    f"https://huggingface.co/{repo}/resolve/{revision}/{relative}",
                    partial,
                    size,
                )
            candidate = path if path.exists() else partial
            with candidate.open("rb") as stream:
                actual = hashlib.file_digest(stream, "sha256").hexdigest()
            if candidate.stat().st_size != size or actual != expected:
                raise ValueError(
                    f"weight integrity mismatch: {relative}; preserve file for diagnosis"
                )
            if candidate == partial:
                partial.rename(path)
            inventory.append(dict(path=str(relative), bytes=size, sha256=actual))
            print(f"Verified {repo}/{relative}", flush=True)
        if not inventory:
            raise ValueError("repository has no safetensors")
        evidence = HERE / ".cache/inventories"
        evidence.mkdir(exist_ok=True)
        stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
        with (evidence / f"{repo.replace('/', '--')}-weights-{stamp}.json").open(
            "x", encoding="utf-8"
        ) as stream:
            json.dump(
                dict(
                    repository=repo,
                    revision=revision,
                    status="verified-not-measured",
                    files=inventory,
                ),
                stream,
                indent=2,
            )
            stream.write("\n")


if __name__ == "__main__":
    main()
