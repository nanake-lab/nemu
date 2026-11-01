"""Local-only native server lifecycle and raw SSE timing for P0-10."""
from __future__ import annotations

from contextlib import AbstractContextManager
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import socket
import subprocess
import time
from types import TracebackType
from typing import Any, BinaryIO
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[2]


def sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def quiet() -> None:
    command = ("$ErrorActionPreference='Stop'; [pscustomobject]@{blocked=@(Get-Process | "
               "Where-Object {$_.ProcessName -in @('Runner.Listener','Runner.Worker','blender')} | "
               "Select-Object ProcessName,Id)} | ConvertTo-Json -Compress")
    result = subprocess.run(["pwsh", "-NoProfile", "-Command", command], check=True,
                            capture_output=True, text=True, encoding="utf-8")
    inventory = json.loads(result.stdout)
    if not isinstance(inventory, dict) or inventory.get("blocked") != []:
        raise RuntimeError("Quiet window absent or cannot be established")


def write_json(path: Path, value: object) -> None:
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


class Server(AbstractContextManager["Server"]):
    def __init__(self, engine: Path, model: Path, output: Path) -> None:
        self.engine, self.model, self.output = engine.resolve(), model.resolve(), output.resolve()
        self.process: subprocess.Popen[bytes] | None = None
        self.watcher: subprocess.Popen[bytes] | None = None
        self.handles: list[BinaryIO] = []
        self.endpoint = ""

    def __enter__(self) -> Server:
        quiet()
        self.output.mkdir(parents=True, exist_ok=False)
        with socket.socket() as port_socket:
            port_socket.bind(("127.0.0.1", 0))
            port = port_socket.getsockname()[1]
        self.endpoint = f"http://127.0.0.1:{port}"
        args = [str(self.engine / "llama-server.exe"), "--log-verbosity", "4", "-m", str(self.model), "--host", "127.0.0.1",
                "--port", str(port), "-ngl", "999", "-c", "8192", "-np", "1", "-b", "512", "-ub", "512",
                "-t", "4", "-tb", "4", "--flash-attn", "off", "--no-cache-prompt", "--no-context-shift",
                "--no-webui", "--no-ui-mcp-proxy", "--offline", "--perf"]
        env = {key: value for key, value in os.environ.items()
               if not key.startswith("LLAMA_") and key != "GGML_BACKEND_PATH"}
        env.update(HF_HUB_OFFLINE="1", HF_HUB_DISABLE_TELEMETRY="1", OMP_NUM_THREADS="4", MKL_NUM_THREADS="4")
        stdout = (self.output / "server.stdout.log").open("xb")
        stderr = (self.output / "server.stderr.log").open("xb")
        self.handles += [stdout, stderr]
        try:
            self.process = subprocess.Popen(args, cwd=self.engine, env=env, stdout=stdout, stderr=stderr,
                                            creationflags=subprocess.CREATE_NO_WINDOW)
            watcher_out = (self.output / "memory-watch.stdout.log").open("xb")
            watcher_err = (self.output / "memory-watch.stderr.log").open("xb")
            self.handles += [watcher_out, watcher_err]
            watcher_args = ["pwsh", "-NoProfile", "-File", str(ROOT / "workers/model-adapter/probe/memory_watch.ps1"),
                            "-OutputPath", str(self.output / "memory.jsonl"), "-StopPath", str(self.output / "stop"),
                            "-RootProcessIds", str(self.process.pid)]
            self.watcher = subprocess.Popen(watcher_args, stdout=watcher_out, stderr=watcher_err,
                                            creationflags=subprocess.CREATE_NO_WINDOW)
            write_json(self.output / "binding.json", {"model": str(self.model), "modelSha256": sha256(self.model),
                "engine": str(self.engine), "arguments": args, "pid": self.process.pid,
                "startedUtc": datetime.now(timezone.utc).isoformat(), "host": "127.0.0.1",
                "strictCoreReceipt": False, "m": 0, "controlVectorInstalled": False})
            deadline = time.monotonic() + 180
            while time.monotonic() < deadline:
                if self.process.poll() is not None:
                    raise RuntimeError("Native server exited before readiness; keep logs")
                try:
                    with urlopen(self.endpoint + "/health", timeout=2) as reply:
                        ready = json.load(reply).get("status") == "ok"
                except (HTTPError, URLError, TimeoutError):
                    ready = False
                if ready:
                    log = (self.output / "server.stderr.log").read_text(encoding="utf-8", errors="replace")
                    offload = re.search(r"offloaded (\d+)/(\d+) layers to GPU", log)
                    if "loaded CUDA backend" not in log or not offload or offload[1] != offload[2]:
                        raise RuntimeError("CUDA/full-offload evidence missing; reject CPU fallback")
                    quiet()
                    return self
                time.sleep(.5)
            raise TimeoutError("Native server readiness timeout")
        except BaseException:
            self.close()
            raise

    def post(self, route: str, body: object) -> dict[str, Any]:
        request = Request(self.endpoint + route, json.dumps(body).encode("utf-8"),
                          {"Content-Type": "application/json"})
        with urlopen(request, timeout=180) as reply:
            result: dict[str, Any] = json.load(reply)
        return result

    def completion(self, payload: dict[str, Any], output: Path) -> dict[str, Any]:
        quiet()
        output.mkdir(parents=True, exist_ok=False)
        write_json(output / "request.json", payload)
        request = Request(self.endpoint + "/completion", json.dumps(payload, ensure_ascii=False).encode("utf-8"),
                          {"Content-Type": "application/json"})
        events = []
        content = ""
        token_times: list[float] = []
        first: float | None = None
        final: dict[str, Any] | None = None
        started = time.perf_counter()
        try:
            with urlopen(request, timeout=180) as reply, (output / "events.jsonl").open("x", encoding="utf-8") as stream:
                for raw in reply:
                    if not raw.startswith(b"data: "):
                        continue
                    elapsed = (time.perf_counter() - started) * 1000
                    data = json.loads(raw[6:])
                    event = {"elapsedMs": elapsed, "data": data}
                    stream.write(json.dumps(event, ensure_ascii=False) + "\n")
                    stream.flush()
                    events.append(event)
                    chunk = data.get("content", "")
                    if chunk:
                        content += chunk
                    tokens = data.get("tokens", [])
                    if tokens:
                        token_times.extend([elapsed] * len(tokens))
                    if first is None and (tokens or chunk):
                        first = elapsed
                    if data.get("stop") is True:
                        final = data
            completed = (time.perf_counter() - started) * 1000
            if final is None:
                raise ValueError("Missing final event")
            quiet()
            # b11669 tokens_cached is the final KV occupancy, not the reused prefix.
            # The request explicitly disables reuse; timings.prompt_n counts processed input.
            processed = final.get("timings", {}).get("prompt_n")
            if (processed != len(payload["prompt"]) or final.get("timings", {}).get("cache_n") != 0
                    or final.get("truncated") is True):
                raise ValueError("Cached/truncated prompt invalidates the registered comparison")
            cumulative = ""
            sentence_ms = None
            for event in events:
                cumulative += event["data"].get("content", "")
                if sentence_ms is None and re.search(r"[。！？!?]", cumulative):
                    sentence_ms = event["elapsedMs"]
            result = {"ttftClientMs": first, "completedClientMs": completed, "content": content,
                      "firstSentenceClientMs": sentence_ms, "clientTokenTimesMs": token_times,
                      "final": final, "eventCount": len(events)}
            write_json(output / "response.json", result)
            return result
        except BaseException as error:
            details = {"errorType": type(error).__name__, "error": str(error),
                       "elapsedMs": (time.perf_counter() - started) * 1000}
            if isinstance(error, HTTPError):
                details["httpBody"] = error.read().decode("utf-8", errors="replace")
            write_json(output / "failure.json", details)
            raise

    def close(self) -> None:
        terminated = self.process is not None and self.process.poll() is None
        if self.process and self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=20)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=10)
        if self.watcher:
            (self.output / "stop").write_text("completed\n", encoding="utf-8")
            self.watcher.wait(timeout=30)
        if self.output.exists():
            write_json(self.output / "shutdown.json", {
                "serverReturnCode": self.process.returncode if self.process else None,
                "watchReturnCode": self.watcher.returncode if self.watcher else None,
                "endedUtc": datetime.now(timezone.utc).isoformat(),
                "serverTerminatedByHarness": terminated})
        for handle in self.handles:
            handle.close()

    def __exit__(self, exc_type: type[BaseException] | None, exc_value: BaseException | None,
                 traceback: TracebackType | None) -> None:
        self.close()
