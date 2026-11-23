"""Sequential, bounded native experiment. Stdlib only; immutable output directories."""
from __future__ import annotations

import argparse
from array import array
import hashlib
import json
import math
from pathlib import Path
import random
import subprocess
import sys
import threading
import time
import traceback
from typing import TextIO

from fixtures import CONTRASTS, cases, choice_prompt, export, final_prompt, tools

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
ENGINE = Path("D:/Dev/cache/P0-10/llama.cpp/b11669")
MODEL = Path("D:/Dev/cache/P0-10/gguf/Qwen--Qwen3-4B-Instruct-2507/cdbee75f17c01a7cc42f958dc650907174af0554/carrier-Q4_K_M.gguf")
MODEL_HASH = "972e3afefe3abcfbc5b38c4dd1e5dcf7244bc3824203b67167b44f4468f129f2"
ARMS = [("zero", "target.f32", 0.0), ("target1", "target.f32", 1.0), ("target4", "target.f32", 4.0)] + [
    (f"random{s}_{a}", f"random{s}.f32", float(a)) for s in (1907, 2718) for a in (1, 4)]


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
        self.timer = threading.Timer(1800, self.process.kill)
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


def extract(probe: Probe) -> None:
    root = probe.output
    delta = [0.0]*2560
    for i, (stem, positive, negative) in enumerate(CONTRASTS):
        for sign, text in [("pos", positive), ("neg", negative)]:
            probe.request(stem + " " + text, activation=f"extract-{i}-{sign}.f32")
        pos, neg = [floats(root / "activations" / f"extract-{i}-{s}.f32") for s in ("pos", "neg")]
        for j in range(2560):
            delta[j] += (pos[j]-neg[j])/len(CONTRASTS)
    metadata = {"target": direction(root / "vectors" / "target.f32", delta)}
    for seed in (1907, 2718):
        rng = random.Random(seed)
        metadata[f"random{seed}"] = direction(root / "vectors" / f"random{seed}.f32", [rng.gauss(0, 1) for _ in range(2560)])
    save(root / "directions.json", metadata)


def guards(probe: Probe) -> None:
    prompt = choice_prompt(cases()[0], 0)
    for name, vector, amp, plain in [("plain", "none", 0, True), ("zero", "target.f32", 0, False),
        ("target", "target.f32", 4, False), ("random", "random1907.f32", 4, False),
        ("repeated", "target.f32", 4, False), ("removed", "none", 0, False)]:
        probe.request(prompt, vector, amp, activation="none" if plain else f"guard-{name}.f32", logits=f"{name}.f32", plain=plain)
    checks = {name: (probe.output / "checks" / f"{name}.f32").read_bytes() for name in ("plain", "zero", "target", "repeated", "removed")}
    result: dict[str, object] = {"zero_equals_plain": checks["zero"] == checks["plain"],
        "removed_equals_plain": checks["removed"] == checks["plain"], "repeated_equals_target": checks["target"] == checks["repeated"]}
    measured: dict[str, float] = {}
    base = floats(probe.output / "activations" / "guard-zero.f32")
    for name, file in [("target", "target.f32"), ("random", "random1907.f32")]:
        active = floats(probe.output / "activations" / f"guard-{name}.f32")
        values = floats(probe.output / "vectors" / file)
        error = max(abs((a-b)-4*v) for a, b, v in zip(active, base, values, strict=True))
        measured[f"{name}_delta_l2"] = math.sqrt(math.fsum((a-b)**2 for a, b in zip(active, base, strict=True)))
        measured[f"{name}_max_add_error"] = error
    save(probe.output / "guards.json", result | measured)
    if not all(result[k] is True for k in ("zero_equals_plain", "removed_equals_plain", "repeated_equals_target")) or any(
        measured[k] > 1e-4 for k in ("target_max_add_error", "random_max_add_error")):
        raise RuntimeError("Injection/cache guard failed; stop without changing protocol")


def experiment(probe: Probe) -> None:
    rows = [(c, order, seed) for c in cases() for order in (0, 1) for seed in (0, 11, 29, 47)]
    random.Random(8301).shuffle(rows)
    with (probe.output / "episodes.jsonl").open("w", encoding="utf-8") as out:
        for index, (case, order, seed) in enumerate(rows):
            arms = list(ARMS)
            random.Random(5000+index).shuffle(arms)
            rng = random.Random(f"{case.id}/{order}/{seed}")
            u_choice, u_final = rng.random(), rng.random()
            for arm, vector, amp in arms:
                first = probe.request(choice_prompt(case, order), vector, amp)
                p = probabilities(first)
                selected = select(p, seed, u_choice)
                action = tools(case, order)[selected][0]
                last = probe.request(final_prompt(case, action), vector, amp, nlabels=2)
                answer = chr(65+select(probabilities(last), seed, u_final))
                relevant_index = next(i for i, t in enumerate(tools(case, order)) if t[0] == "relevant")
                record = {"case": case.id, "family": case.family, "missing": case.missing, "order": order, "seed": seed,
                    "arm": arm, "u_choice": u_choice, "u_final": u_final, "choice": chr(65+selected), "action": action,
                    "p_relevant": p[relevant_index], "answer": answer, "expected": case.correct, "correct": answer == case.correct,
                    "score": 10*int(answer == case.correct)-int(action != "submit"), "first": first, "last": last}
                out.write(json.dumps(record, allow_nan=False) + "\n")
                out.flush()
            if index % 12 == 0:
                print(json.dumps({"completed_episodes": (index+1)*len(ARMS), "total": len(rows)*len(ARMS)}), flush=True)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--executable", type=Path, required=True)
    parser.add_argument("--quiet-window-confirmed", action="store_true")
    args = parser.parse_args()
    if not args.quiet_window_confirmed:
        raise SystemExit("Requires the user's quiet window and stopped runner")
    if sys.byteorder != "little":
        raise SystemExit("Expected Windows little-endian float evidence")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    for folder in ("prompts", "vectors", "activations", "checks"):
        (output / folder).mkdir()
    started = time.time()
    try:
        if sha(MODEL) != MODEL_HASH:
            raise RuntimeError("Model differs from P0-10")
        inputs = list(HERE.glob("*.py")) + [HERE / "choice_probe.cpp", REPO / "spikes/carrier-engine/native_probe.hpp", args.executable]
        binding = {"started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(started)), "model": str(MODEL),
            "model_sha256": MODEL_HASH, "python": sys.version, "inputs": {str(p): sha(p) for p in inputs},
            "engine_dlls": {p.name: sha(p) for p in ENGINE.glob("*.dll")}, "arms": ARMS}
        save(output / "binding.json", binding)
        save(output / "fixtures.json", export())
        with (output / "native-stderr.log").open("w", encoding="utf-8") as err:
            probe = Probe(args.executable.resolve(), output, err)
            try:
                save(output / "label-token-ids.json", probe.labels)
                extract(probe)
                guards(probe)
                experiment(probe)
            finally:
                probe.close()
        save(output / "exit.json", {"status": "completed", "wall_seconds": time.time()-started})
    except BaseException:
        (output / "failure.txt").write_text(traceback.format_exc(), encoding="utf-8")
        save(output / "exit.json", {"status": "failed", "wall_seconds": time.time()-started})
        raise


if __name__ == "__main__":
    main()
