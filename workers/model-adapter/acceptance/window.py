"""One exclusive P5-13 window. Run from the adapter/P5-07 shared test environment."""
from __future__ import annotations

import argparse
from contextlib import ExitStack, contextmanager
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from typing import Iterator
from uuid import uuid4

from suite import PACKAGE, quiet


def utc() -> str:
    return datetime.now(timezone.utc).isoformat()


@contextmanager
def lease(cache: Path) -> Iterator[None]:
    flag = cache / "gpu-window.txt"
    owner = f"P5-13 {utc()} {uuid4()}\n"
    with flag.open("x", encoding="utf-8", newline="") as f:
        f.write(owner)
    try:
        yield
    finally:
        if flag.read_text("utf-8") == owner:
            flag.unlink()


@contextmanager
def child(cmd: list[str], *, env: dict[str, str] | None = None,
          log: Path | None = None) -> Iterator[subprocess.Popen[bytes]]:
    with ExitStack() as stack:
        target = stack.enter_context(log.open("xb")) if log else subprocess.DEVNULL
        p = subprocess.Popen(cmd, stdout=target, stderr=subprocess.STDOUT,
                             creationflags=subprocess.CREATE_NO_WINDOW, env=env)
        try:
            yield p
        finally:
            if p.poll() is None:
                subprocess.run(["taskkill", "/pid", str(p.pid), "/t", "/f"],
                               stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)
                p.wait(timeout=30)


def main() -> None:
    parser = argparse.ArgumentParser()
    for name in ("out", "engine", "model", "tokenizer", "mmproj", "shim", "voice-python",
                 "voice-source", "nltk", "ffmpeg", "browser-cache"):
        parser.add_argument("--" + name, required=True, type=Path)
    a = parser.parse_args()
    quiet()
    for status in ("queued", "in_progress", "waiting", "requested", "pending"):
        data = subprocess.check_output(["gh", "api", "repos/nanake-lab/nemu/actions/workflows/windows.yml/runs"
                                         + "?status=" + status + "&per_page=1"], text=True, encoding="utf-8")
        if json.loads(data)["total_count"]:
            raise RuntimeError("Windows workflow " + status)
    a.out.mkdir(parents=True, exist_ok=False)
    log = a.out / "window.jsonl"
    def mark(event: str, **data: object) -> None:
        with log.open("a", encoding="utf-8") as f:
            f.write(json.dumps(dict(utc=utc(), event=event, **data)) + "\n")
    end = time.monotonic() + 7200  # User-facing estimate and total GPU lease.
    def guard() -> None:
        quiet()
        if time.monotonic() >= end:
            raise TimeoutError("window time limit")
    def group(name: str, directory: str, companions: tuple[subprocess.Popen[bytes], ...] = ()) -> None:
        guard()
        cmd = [sys.executable, str(Path(__file__).with_name("suite.py")), "--group", name,
               "--out", str(a.out / directory)]
        for key in ("engine", "model", "tokenizer", "mmproj", "shim"):
            cmd.extend(["--" + key, str(getattr(a, key))])
        mark("group_start", group=directory)
        with child(cmd, log=a.out / (directory + ".log")) as p:
            while p.poll() is None:
                if any(q.poll() is not None for q in companions):
                    raise RuntimeError("resident or monitoring process exited")
                guard()
                time.sleep(2)
            mark("group_end", group=directory, code=p.returncode)
            if p.returncode:
                raise RuntimeError("group failed: " + directory)
    def ready(flag: Path, p: subprocess.Popen[bytes]) -> None:
        until = time.monotonic() + 180  # Startup allowance only; never a model latency threshold.
        while not flag.exists():
            guard()
            if p.poll() is not None or time.monotonic() >= until:
                raise RuntimeError("resident fixture did not become ready: " + flag.name)
            time.sleep(1)
    def sampled(name: str, directory: str, companions: tuple[subprocess.Popen[bytes], ...] = ()) -> None:
        stop = a.out / (directory + ".stop")
        with child(["pwsh", "-NoProfile", "-File", str(Path(__file__).with_name("memory.ps1")),
                    "-MeasurementRoot", str(os.getpid()), "-OutputPath", str(a.out / (directory + "-wddm.jsonl")),
                    "-StopPath", str(stop)]) as monitor:
            try:
                group(name, directory, (*companions, monitor))
            finally:
                stop.touch()
                monitor.wait(timeout=30)
                if monitor.returncode:
                    raise RuntimeError("WDDM sampling failed")
    with lease(Path("D:/Dev/cache")):
        mark("open")
        try:
            group("path", "path")
            sampled("irl", "mode-b-irl")
            group("identity", "identity")
            stage, voice = a.out / "stage-ready.json", a.out / "voice-ready.json"
            stop = a.out / "resident.stop"
            env = dict(os.environ, PLAYWRIGHT_BROWSERS_PATH=str(a.browser_cache))
            with child([shutil.which("node") or "node", str(PACKAGE / "probe/render_load.mjs"),
                        str(stage), str(stop)], env=env) as render:
                ready(stage, render)
                with child([str(a.voice_python), str(Path(__file__).with_name("resident_voice.py")),
                            "--source", str(a.voice_source), "--nltk", str(a.nltk), "--ffmpeg", str(a.ffmpeg),
                            "--ready", str(voice), "--stop", str(stop)]) as tts:
                    try:
                        ready(voice, tts)
                        for i in range(1, 4):
                            sampled("latency", f"joint-{i}", (render, tts))
                        sampled("mode-b", "mode-b-joint", (render, tts))
                    finally:
                        stop.touch()
                        if tts.poll() is None:
                            tts.wait(timeout=30)
                render.wait(timeout=30)
            mark("finished")
        except BaseException as exc:
            mark("failed", reason=str(exc))
            raise
        finally:
            mark("close")


if __name__ == "__main__":
    main()
