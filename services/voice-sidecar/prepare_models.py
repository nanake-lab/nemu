"""Explicit setup only: pin HF file digests or download and verify an existing lock.

No model downloads occur during service startup or CI tests.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
from typing import cast
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
VARIANTS = {"0.6B": ("Qwen/Qwen3-TTS-12Hz-0.6B-Base", "5d83992436eae1d760afd27aff78a71d676296fc"),
            "1.7B": ("Qwen/Qwen3-TTS-12Hz-1.7B-Base", "main"),
            "VoiceDesign": ("Qwen/Qwen3-TTS-12Hz-1.7B-VoiceDesign", "main")}


def fetch(url: str) -> bytes:
    with urllib.request.urlopen(url, timeout=60) as response:
        return cast(bytes, response.read())


def pin() -> None:
    path = ROOT / "config/models.lock.json"
    lock = json.loads(path.read_text(encoding="utf-8-sig"))
    models = {}
    for variant, (repo, revision) in VARIANTS.items():
        info = json.loads(fetch(f"https://huggingface.co/api/models/{repo}/revision/{revision}?blobs=true"))
        revision = info["sha"]
        evidence = ROOT / "reports/P4-02" / f"hf-{variant}-metadata.json"
        evidence.write_text(json.dumps(info, indent=2) + "\n", encoding="utf-8")
        files = []
        for item in info["siblings"]:
            name = item["rfilename"]
            if name.startswith(".") or name == "README.md":
                continue
            digest = item.get("lfs", {}).get("sha256")
            if digest is None:
                digest = hashlib.sha256(fetch(f"https://huggingface.co/{repo}/resolve/{revision}/{name}")).hexdigest()
            files.append({"path": name, "size": item["size"], "sha256": digest})
        models[variant] = {"id": repo, "revision": revision, "license": "Apache-2.0",
                           "source": f"https://huggingface.co/{repo}", "redistributable": True,
                           "files": files}
        print(f"Pinned {repo}@{revision}: {len(files)} files", flush=True)
    lock["tts"] = {"version": 1, "models": models}
    path.write_text(json.dumps(lock, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def download(cache: Path, variant: str) -> None:
    lock = json.loads((ROOT / "config/models.lock.json").read_text(encoding="utf-8-sig"))
    model = lock["tts"]["models"][variant]
    for item in model["files"]:
        target = cache / model["id"].replace("/", "--") / model["revision"] / item["path"]
        if target.is_file():
            with target.open("rb") as stream:
                if hashlib.file_digest(stream, "sha256").hexdigest() == item["sha256"]:
                    print(f"Verified cached {item['path']}", flush=True)
                    continue
        target.parent.mkdir(parents=True, exist_ok=True)
        partial = target.with_suffix(target.suffix + ".partial")
        url = f"{model['source']}/resolve/{model['revision']}/{item['path']}"
        if item["size"] > 64 * 1024 * 1024:
            # Reuse the repository's already verified bounded-range downloader.
            spec = importlib.util.spec_from_file_location(
                "nemu_probe_download", ROOT / "workers/model-adapter/probe/download_weights.py")
            if spec is None or spec.loader is None:
                raise RuntimeError("Probe downloader unavailable")
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            module.download_chunks("curl.exe", url, partial, item["size"])
        else:
            subprocess.run(["curl.exe", "--fail", "--location", "--silent", "--show-error",
                            "--retry", "3", "--connect-timeout", "30", "--max-time", "1800",
                            "--output", str(partial), url], check=True)
        with partial.open("rb") as stream:
            digest = hashlib.file_digest(stream, "sha256").hexdigest()
        if partial.stat().st_size != item["size"] or digest != item["sha256"]:
            raise ValueError("MODEL_HASH_MISMATCH; partial retained")
        partial.replace(target)
        print(f"Downloaded and verified {item['path']}", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pin", action="store_true")
    parser.add_argument("--cache", type=Path)
    parser.add_argument("--variant", choices=list(VARIANTS))
    args = parser.parse_args()
    if args.pin:
        pin()
    elif args.cache and args.variant:
        download(args.cache, args.variant)
    else:
        parser.error("provide --pin or --cache and --variant")
