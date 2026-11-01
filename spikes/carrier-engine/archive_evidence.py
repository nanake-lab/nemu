"""Losslessly deduplicate frozen experiment files; verify every gzip roundtrip."""
from __future__ import annotations

import argparse
import gzip
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
from typing import Any

from http_probe import sha256, write_json


def archive(source: Path, inventory: Path, output: Path) -> dict[str, Any]:
    original = json.loads(inventory.read_text(encoding="utf-8-sig"))
    entries = original["files"]
    if output.exists():
        raise FileExistsError("Preserve existing archive")
    output.mkdir(parents=True)
    (output / "objects").mkdir()
    objects: dict[str, dict[str, Any]] = {}
    files = []
    for index, entry in enumerate(entries):
        relative = PurePosixPath(entry["path"])
        digest = entry["sha256"].lower()
        if relative.is_absolute() or ".." in relative.parts or "\\" in str(relative) or ":" in str(relative):
            raise ValueError("Unsafe evidence path")
        if not re.fullmatch("[0-9a-f]{64}", digest) or entry.get("hashError"):
            raise ValueError("Incomplete original hash inventory")
        path = source.joinpath(*relative.parts)
        if not path.resolve().is_relative_to(source.resolve()):
            raise ValueError("Evidence path escaped source")
        data = path.read_bytes()
        if len(data) != entry["bytes"] or hashlib.sha256(data).hexdigest() != digest:
            raise ValueError("Frozen evidence changed: " + str(relative))
        object_name = "objects/" + digest + ".gz"
        if digest not in objects:
            packed = gzip.compress(data, compresslevel=6, mtime=0)
            with (output / object_name).open("xb") as stream:
                stream.write(packed)
            restored = gzip.decompress((output / object_name).read_bytes())
            if restored != data:
                raise ValueError("Lossless archive verification failed")
            objects[digest] = {"path": object_name, "bytes": len(packed),
                               "sha256": hashlib.sha256(packed).hexdigest()}
        files.append({"path": str(relative), "bytes": len(data), "sha256": digest, "object": object_name})
        if (index + 1) % 1000 == 0:
            print(f"Verified and archived {index + 1}/{len(entries)} original files", flush=True)
    result = {"format": "sha256-deduplicated-gzip-v1", "source": str(source),
        "sourceInventorySha256": sha256(inventory), "methodSha256": sha256(Path(__file__)),
        "fileCount": len(files), "originalBytes": sum(f["bytes"] for f in files),
        "uniqueFiles": len(objects), "packedBytes": sum(o["bytes"] for o in objects.values()),
        "verifiedRoundtrip": True, "files": files, "objects": objects}
    write_json(output / "raw-filemap.json", result)
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--inventory", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = archive(args.source.resolve(), args.inventory.resolve(), args.output.resolve())
    print(json.dumps({key: result[key] for key in ["fileCount", "originalBytes", "uniqueFiles", "packedBytes", "verifiedRoundtrip"]}))


if __name__ == "__main__":
    main()
