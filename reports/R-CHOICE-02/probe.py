"""Sequential, bounded native experiment. Stdlib only; immutable output directories."""
from __future__ import annotations

from array import array
import hashlib
import json
import math
from pathlib import Path
import subprocess
import threading
import time
from typing import TextIO



HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
ENGINE = Path("D:/Dev/cache/P0-10/llama.cpp/b11669")
MODEL = Path("D:/Dev/cache/P0-10/gguf/Qwen--Qwen3-4B-Instruct-2507/cdbee75f17c01a7cc42f958dc650907174af0554/carrier-Q4_K_M.gguf")
MODEL_HASH = "972e3afefe3abcfbc5b38c4dd1e5dcf7244bc3824203b67167b44f4468f129f2"
def sha(path: Path) -> str:
    with path.open("rb") as f:
        return hashlib.file_digest(f, "sha256").hexdigest()


def save(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False) + "\n", encoding="utf-8")


def floats(path: Path) -> list[float]:
    v = array("f")
    v.frombytes(path.read_bytes())
    return list(v)


def direction(path: Path, values: list[float]) -> dict[str, object]:
    norm = math.sqrt(math.fsum(x*x for x in values))
    if not math.isfinite(norm) or norm <= 0:
        raise ValueError("Invalid extracted direction")
    v = array("f", [x / norm for x in values])
    path.write_bytes(v.tobytes())
    return {"raw_norm": norm, "saved_norm": math.sqrt(math.fsum(x*x for x in v)), "sha256": sha(path)}


class Probe:
    def __init__(self, executable: Path, output: Path, stderr: TextIO) -> None:
        self.output = output
        self.process = subprocess.Popen([str(executable), str(ENGINE), str(MODEL), str(output), "--quiet-window-confirmed"],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, text=True, encoding="utf-8", bufsize=1)
        self.timer = threading.Timer(600, self.process.kill)
        self.timer.daemon = True
        self.timer.start()
        self.requests = (output / "requests.jsonl").open("w", encoding="utf-8")
        self.extra = (output / "native-stdout.log").open("w", encoding="utf-8")
        try:
            self.labels = [int(x) for x in self.receive("READY").split("\t")]
        except BaseException:
            self.timer.cancel()
            if self.process.poll() is None:
                self.process.kill()
            self.process.wait()
            self.requests.close()
            self.extra.close()
            raise

    def receive(self, kind: str) -> str:
        assert self.process.stdout is not None
        while True:
            line = self.process.stdout.readline()
            if not line:
                raise RuntimeError(f"Native process ended before {kind}: {self.process.poll()}")
            if line.startswith(f"RCHOICE {kind}\t"):
                return line.rstrip("\n").split("\t", 1)[1]
            self.extra.write(line)
            self.extra.flush()

    def request(self, prompt: str, vector: str = "none", amplitude: float = 0, nlabels: int = 3,
                activation: str = "none", logits: str = "none", plain: bool = False) -> dict[str, object]:
        name = hashlib.sha256(prompt.encode()).hexdigest() + ".txt"
        path = self.output / "prompts" / name
        if not path.exists():
            path.write_bytes(prompt.encode("utf-8"))
        command = "\t".join(["P" if plain else "X", name, vector, str(amplitude), str(nlabels), activation, logits])
        assert self.process.stdin is not None
        started = time.monotonic()
        self.process.stdin.write(command + "\n")
        self.process.stdin.flush()
        fields = self.receive("OK").split("\t")
        record: dict[str, object] = {"prompt": name, "vector": vector, "amplitude": amplitude, "labels": nlabels,
            "plain": plain, "activation": activation, "full_logits": logits, "input_tokens": int(fields[0]),
            "unconstrained_argmax": int(fields[1]), "logits": [float(x) for x in fields[2:2+nlabels]],
            "allowed_mass": float(fields[5]), "callback_count": int(fields[6]), "wall_seconds": time.monotonic()-started}
        self.requests.write(json.dumps(record, allow_nan=False) + "\n")
        self.requests.flush()
        return record

    def close(self) -> None:
        try:
            if self.process.poll() is None:
                assert self.process.stdin is not None
                self.process.stdin.write("QUIT\n")
                self.process.stdin.flush()
                self.process.wait(timeout=30)
        finally:
            if self.process.poll() is None:
                self.process.kill()
                self.process.wait()
            self.timer.cancel()
            self.requests.close()
            self.extra.close()
        if self.process.returncode != 0:
            raise RuntimeError(f"Native exit {self.process.returncode}")


def probabilities(record: dict[str, object]) -> list[float]:
    raw = record["logits"]
    assert isinstance(raw, list)
    values = [float(x) for x in raw]
    weights = [math.exp((x-max(values))/0.7) for x in values]
    return [w/sum(weights) for w in weights]


def select(p: list[float], seed: int, u: float) -> int:
    if seed == 0:
        return max(range(len(p)), key=lambda i: p[i])
    total = 0.0
    for i, v in enumerate(p):
        total += v
        if u < total:
            return i
    return len(p)-1


