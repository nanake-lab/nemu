"""Local native process; GPU is impossible through the offline preparation path."""
from __future__ import annotations

import math
from pathlib import Path
import random
import subprocess
import threading

from io_support import digest, save

ENGINE = Path("D:/Dev/cache/P0-10/llama.cpp/b11669")
MODEL = Path("D:/Dev/cache/P0-10/gguf/Qwen--Qwen3-4B-Instruct-2507/cdbee75f17c01a7cc42f958dc650907174af0554/carrier-Q4_K_M.gguf")
MODEL_SHA = "972e3afefe3abcfbc5b38c4dd1e5dcf7244bc3824203b67167b44f4468f129f2"
DLL_SHA = "aacb073c9c0d30090dd3e56f49772545452ccc4fd311efbfdb7d1bba7bbb0bda"


class Carrier:
    def __init__(self, executable: Path, root: Path, quiet_confirmed: bool) -> None:
        if not quiet_confirmed:
            raise ValueError("This run needs an explicit human quiet-window confirmation")
        if digest(MODEL) != MODEL_SHA or digest(ENGINE / "llama.dll") != DLL_SHA:
            raise ValueError("Pinned carrier mismatch")
        self.root, self.index = root, 0
        save(root / "carrier-binding.json", {"model_sha256": MODEL_SHA, "llama_dll_sha256": DLL_SHA,
            "executable_sha256": digest(executable), "m": 0, "context": 8192,
            "prompt_limit": 4096, "temperature": 0.7, "KV": "clear before every request"})
        self.stderr = (root / "native.stderr.log").open("xb")
        self.process = subprocess.Popen([str(executable), str(ENGINE), str(MODEL), str(root),
            "--quiet-window-confirmed"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.stderr,
            text=True, encoding="utf-8", creationflags=subprocess.CREATE_NO_WINDOW)
        self.timer = threading.Timer(5400, self.process.kill)
        self.timer.daemon = True
        self.timer.start()
        assert self.process.stdout is not None
        if self.process.stdout.readline().strip() != "READY":
            self.close()
            raise RuntimeError("Native startup failed; preserve evidence")

    def choose(self, prompt: str, rng: random.Random) -> tuple[str, dict[str, object]]:
        name = f"prompt-{self.index:05d}.txt"
        with (self.root / name).open("x", encoding="utf-8", newline="\n") as stream:
            stream.write(prompt)
        assert self.process.stdin is not None and self.process.stdout is not None
        self.process.stdin.write(name + "\n")
        self.process.stdin.flush()
        raw = self.process.stdout.readline().strip()
        fields = raw.split("\t")
        if len(fields) != 7:
            raise RuntimeError("Native request failed; no automatic retry")
        pa, pb = float(fields[1]), float(fields[2])
        if not all(math.isfinite(x) and 0 <= x <= 1 for x in (pa, pb)) or pa + pb > 1 + 1e-10:
            raise ValueError("Invalid probabilities")
        uniform = rng.random()
        selected = "A" if uniform < pa else "B" if uniform < pa + pb else "INVALID"
        result: dict[str, object] = {"prompt": name, "raw": raw, "p_A_full": pa,
            "p_B_full": pb, "valid_mass": pa + pb, "uniform": uniform,
            "selected": selected, "unrestricted_argmax": fields[6], "m": 0}
        save(self.root / f"decision-{self.index:05d}.json", result)
        self.index += 1
        return selected, result

    def close(self) -> None:
        self.timer.cancel()
        if self.process.poll() is None:
            if self.process.stdin:
                try:
                    self.process.stdin.write("QUIT\n")
                    self.process.stdin.flush()
                except OSError:
                    pass
            try:
                self.process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()
        self.stderr.close()
