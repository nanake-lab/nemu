"""Serial process orchestrator: config -> independent Blender stages -> asset-lint."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
from typing import Any

from config import load_config, writable_path
from errors import fail

BASE = Path(__file__).resolve().parent
ROOT = BASE.parent.parent
STEPS = {
    "import": "import_vrm.py", "validate": "validate_source.py", "tail": "build_tail.py",
    "springs": "springs.py", "expressions": "expressions.py", "materials": "materials.py",
    "textures": "textures.py", "export": "export_vrm.py", "profile": "profile.py",
}


def blender_path() -> str:
    env = os.environ.get("NEMU_BLENDER")
    if env:
        if not Path(env).is_file():
            fail("version", "NEMU_BLENDER", "existing executable", env)
        return env
    found = sorted(Path("C:/Program Files/Blender Foundation").glob("*/blender.exe"))
    if len(found) != 1:
        fail("version", "Blender", "NEMU_BLENDER or exactly one installed executable", found)
    return str(found[0])


def process(command: list[str], log: Path, step: str) -> int:
    with log.open("w", encoding="utf-8") as stream:
        stream.write(json.dumps({"step": step, "command": command}, ensure_ascii=False) + "\n")
        stream.flush()
        result = subprocess.run(command, cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT, check=False)
    print(f"step={step} exit={result.returncode} log={log}", flush=True)
    if result.returncode:
        print(log.read_text(encoding="utf-8", errors="replace")[-8000:], file=sys.stderr)
    return result.returncode


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", type=Path, default=ROOT / "content/nemu/build.yaml")
    parser.add_argument("--output", type=Path, default=BASE / ".cache/build")
    parser.add_argument("--step", choices=list(STEPS))
    parser.add_argument("paths", nargs="*", type=Path)
    args = parser.parse_args()
    output = writable_path(args.output, ROOT)
    if args.step and (len(args.paths) != 2 or not args.paths[0].is_file()):
        fail(args.step, "paths", "existing input.blend and output.blend", args.paths)
    if not args.step and args.paths:
        fail("config", "arguments", "no positional paths for full build", args.paths)
    config = load_config(args.config.resolve())
    source = (ROOT / config["source"]).resolve()
    if not source.is_file():
        fail("import", str(source), "existing source VRM", "missing")
    config["source"] = str(source)
    if source == output / "nemu.vrm":
        fail("config", str(source), "source distinct from output", "would overwrite source")
    output.mkdir(parents=True, exist_ok=True)
    cache = output / ".cache"
    cache.mkdir(exist_ok=True)
    # One pipeline/asset:step process at a time, even when output directories differ.
    lock = BASE / ".cache/pipeline.lock"
    lock.parent.mkdir(exist_ok=True)
    try:
        handle = lock.open("x", encoding="utf-8")
    except FileExistsError:
        fail("schedule", str(lock), "exclusive Blender window (remove stale lock only after checking PID)", "another run or stale lock")
    try:
        with handle:
            handle.write(str(os.getpid()))
        normalized = cache / "resolved-build.json"
        normalized.write_text(json.dumps(config, ensure_ascii=False, allow_nan=False), encoding="utf-8")
        blender = blender_path()
        previous: Path | None = None
        for index, (step, script) in enumerate(STEPS.items(), 1):
            if args.step and args.step != step:
                continue
            target = writable_path(args.paths[1], ROOT) if args.step else cache / f"{index:02d}-{step}.blend"
            if args.step and target == args.paths[0].resolve():
                fail(step, str(target), "output distinct from input", "same path")
            target.parent.mkdir(parents=True, exist_ok=True)
            command = [blender, "-b"]
            if args.step:
                command.append(str(args.paths[0].resolve()))
            elif previous:
                command.append(str(previous))
            command += ["--python-exit-code", "1", "--python", str(BASE / script), "--", "--config", str(normalized), "--output-dir", str(output), "--output-blend", str(target)]
            if process(command, output / f"{index:02d}-{step}.log", step):
                fail(step, script, "exit code 0", "failed; see step log")
            if not target.is_file():
                fail(step, str(target), "saved intermediate blend", "missing")
            previous = target
        if args.step:
            return
        manifest: list[dict[str, Any]] = [{"path": "nemu.vrm", "kind": "model", **config["provenance"]}]
        (output / "manifest.json").write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        pnpm = shutil.which("pnpm.cmd" if os.name == "nt" else "pnpm")
        if pnpm is None:
            fail("lint", "pnpm", "installed executable", "missing")
        command = [pnpm, "asset:lint", str(output / "nemu.vrm"), "--profile", str(output / "nemu.profile.json"), "--manifest", str(output / "manifest.json"), "--json"]
        code = process(command, output / "10-lint.log", "lint")
        # Do not change asset-lint or its exit code. Sample exceptions are evidence only.
        text = (output / "10-lint.log").read_text(encoding="utf-8")
        start = text.find('{\n  "file"')
        if start >= 0:
            lint, _ = json.JSONDecoder().raw_decode(text[start:])
            (output / "lint.json").write_text(json.dumps(lint, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        if code:
            fail("lint", "nemu.vrm", "asset-lint 0 errors", f"exit {code}; see raw diagnostics (sample exceptions are reviewed separately)")
    finally:
        lock.unlink()


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)
