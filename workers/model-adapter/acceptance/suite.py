"""P5-13 GPU measurements. Run only inside the user-arranged exclusive window."""
from __future__ import annotations

import argparse
import asyncio
import ctypes as ct
from dataclasses import replace
from datetime import datetime, timezone
import hashlib
import importlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
from time import perf_counter
from typing import Any

ROOT = Path(__file__).resolve().parents[3]
PACKAGE = ROOT / "workers/model-adapter"
sys.path[:0] = [str(PACKAGE / "src"), str(PACKAGE / "generated"), str(PACKAGE / "tests"),
                str(ROOT / "core/contracts"), str(ROOT / "workers/irl/src")]
from echo.typed_api import to_value  # noqa: E402
from echo.types import (FactProjection, JActivationDtype, JDecodeMask, JLayerKind,  # noqa: E402
                        JPrefillMask, JResidualStage, TaskInput, UserText)
from nemu_model_adapter import Assets, LocalCarrier, Unmodulated  # noqa: E402
from nemu_model_adapter.carrier import chat_prompt  # noqa: E402
from nemu_model_adapter.native import Native, sha256_file  # noqa: E402
from nemu_model_adapter.resources import canonical_ref  # noqa: E402
from test_resources import LIMITS, fixture, publication  # noqa: E402
from verify_j import rebind  # noqa: E402

# Synthetic identities/axes for the experiment lineage only. P0-12 mapped layer 15.
# Packet coefficients stay inside the pre-existing P5-42 fixture's per-column budget.
FACT = "Your name is Nemu. You are the Nemu project's Windows desktop companion."
SENTENCE = "今天的努力值得肯定请带着信心继续向前走吧。"
QUERY = "请逐字输出下面这句话，不添加其他内容：" + SENTENCE
CORPUS = {"irl": ("irl-inputs-v1.jsonl", "da05c8d65dffc593d22dc4b5560f2601a3790ae736b99b86789c7974a8a7a3bf"),
          "identity": ("identity-v1.jsonl", "8fdedd1c9e013c840e7cba4f2b8fcbabd6d0da14a0ce0c57fac8ef34eda238ca")}


def quiet() -> None:
    # Filter at collection. No enumeration/recording of unrelated programs.
    for name in ("Runner.Listener.exe", "Runner.Worker.exe"):
        proc = subprocess.run(["tasklist", "/FI", "IMAGENAME eq " + name, "/FO", "CSV", "/NH"],
                              capture_output=True, check=True)
        if name.lower().encode() in proc.stdout.lower():
            raise RuntimeError("runner active")


def rows(name: str) -> list[dict[str, Any]]:
    filename, expected = CORPUS[name]
    path = PACKAGE / "tests/data" / filename
    if sha256_file(path).hex() != expected:
        raise ValueError("frozen input hash")
    return [json.loads(line) for line in path.read_text("utf-8").splitlines()]


def task_for(text: str, protocol: str, facts: FactProjection) -> TaskInput:
    user = canonical_ref("UserText", to_value(UserText(1, text)), LIMITS)
    return TaskInput(1, user, "test-facts-v1", protocol, facts, "synthetic-0",
                     96, 30_000_000_000)  # P0-07 speech fixture budget


def bundle(assets: Assets, arm: str, *, learned: bool = True) -> tuple[Any, ...]:
    b = fixture()
    raw = b["direction_set"]["hook"]
    pins = json.loads((PACKAGE / "src/nemu_model_adapter/manifest.json").read_bytes())
    raw.update(layer_index=15, width=2560, layer_kind=int(JLayerKind.J_ATTENTION_LAYER),
               residual_stage=int(JResidualStage.J_BLOCK_RESIDUAL_OUTPUT),
               dtype=int(JActivationDtype.J_FLOAT32), prefill_mask=int(JPrefillMask.J_PREFILL_ALL),
               decode_mask=int(JDecodeMask.J_DECODE_ALL))
    for key, identity, length in (("model", pins["gguf"]["carrier-Q4_K_M.gguf"], assets.model.stat().st_size),
                                  ("tokenizer", pins["tokenizer_sha256"], assets.tokenizer.stat().st_size),
                                  ("carrier", assets.shim_sha256.hex(), assets.shim.stat().st_size)):
        raw[key].update(digest=identity, byte_length=str(length))
        b["receipt"][key] = raw[key]
    for i, vector in enumerate(b["vectors"]):
        vector["values"] = [float(j == i) for j in range(raw["width"])]
    if not learned:
        for collection in (b["vectors"], b["validation"]["directions"], b["direction_set"]["directions"]):
            collection.pop()
        b["receipt"]["directions"] = b["receipt"]["directions"][:4]
        b["receipt"]["total_bound"] = 0.5
        b["direction_set"]["version"] = "synthetic-four"
    for use in b["receipt"]["directions"]:
        if use["status"] == 1:
            use["coefficient"] *= {"zero": 0, "plus": 1, "minus": -1}[arm]
    rebind(b)
    return publication(b)


class Evidence:
    def __init__(self, root: Path) -> None:
        root.mkdir(parents=True, exist_ok=False)
        self.root = root

    def add(self, name: str, **values: Any) -> None:
        with (self.root / (name + ".jsonl")).open("a", encoding="utf-8", newline="\n") as f:
            f.write(json.dumps(dict(utc=datetime.now(timezone.utc).isoformat(), **values),
                               ensure_ascii=False, allow_nan=False) + "\n")


def logits_hash(native: Native, handle: int) -> str:
    logits = native.lib.carrier_logits
    logits.restype = ct.POINTER(ct.c_float)
    logits.argtypes = [ct.c_void_p, ct.POINTER(ct.c_int)]
    count = ct.c_int()
    ptr = logits(handle, ct.byref(count))
    if not ptr or count.value != 248320:
        raise RuntimeError("full logits unavailable")
    return hashlib.sha256(ct.string_at(ptr, count.value * 4)).hexdigest()


def path_checks(a: argparse.Namespace, assets: Assets, ev: Evidence) -> None:
    """Compare full logits for each greedy position; hashes are over native F32 bytes."""
    prompts = [chat_prompt(None, s) for s in (QUERY, "Please introduce yourself.", "お名前は？")]
    trajectories: dict[tuple[str, int], list[str]] = {}
    for build, arms in (("plain", ["zero"]), ("carrier", ["zero", "plus", "minus", "plus-again", "zero-again"]),
                        ("drop-vector", ["plus"])):
        quiet()
        shim = assets.shim if build == "carrier" else assets.shim.with_name("measurement-" + build + ".dll")
        native = Native(replace(assets, shim=shim, shim_sha256=sha256_file(shim)), load_mmproj=False)
        try:
            for arm in arms:
                state = arm.split("-")[0]
                for i, prompt in enumerate(prompts):
                    quiet()
                    delta = tuple(({"zero": 0, "plus": 1, "minus": -1}[state] if j == 0 else 0)
                                  for j in range(2560))  # P0-12 integer e0 diagnostic
                    try:
                        h = native.begin(prompt, 15, delta, None, 24, 30_000_000_000)
                    except Unmodulated as exc:
                        ev.add("path", build=build, arm=arm, language_index=i, rejected=str(exc))
                        if build == "drop-vector" and "AT13" in str(exc):
                            continue
                        raise
                    try:
                        sequence = []
                        for _ in range(24):
                            piece = native.next(h)
                            sequence.append(logits_hash(native, h))
                            if piece is None:
                                break
                        stats = native.stats(h)
                        ev.add("path", build=build, arm=arm, language_index=i, logits_sha256=sequence,
                               positions=stats.positions, batches=stats.batches, max_abs_error=stats.max_abs_error,
                               max_delta_l2=stats.max_delta_l2)
                        if build == "drop-vector":
                            raise AssertionError("AT13 false success")
                        key = state, i
                        if key in trajectories and trajectories[key] != sequence:
                            raise AssertionError("fresh or unhooked logits mismatch")
                        if state != "zero" and sequence == trajectories["zero", i]:
                            raise AssertionError("nonzero arm identical to baseline")
                        trajectories[key] = sequence
                    finally:
                        native.end(h)
        finally:
            native.close()


async def run(a: argparse.Namespace, assets: Assets, ev: Evidence) -> None:
    packets = {arm: bundle(assets, arm) for arm in ("zero", "plus", "minus")}
    e, r, p, hook, obj = packets["zero"]
    # Reuse P5-07's synthetic view construction, original wire schema and real consumer.
    worker_test = ROOT / "workers/irl/tests/test_worker.py"
    spec = importlib.util.spec_from_file_location("irl_inputs", worker_test)
    assert spec and spec.loader
    views = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(views)
    worker = importlib.import_module("nemu_irl.worker")
    validator = importlib.import_module("jsonschema").Draft202012Validator(worker.wire_schema())
    facts = views.view_for().c_core
    carrier = LocalCarrier(assets, hook, load_mmproj=True)
    try:
        if a.group == "resources":
            # Same prompt, same model instance. Remove the learned column, then restore
            # it. Observe actual prefill logits and physical additions at the public API.
            evidence = {}
            for learned in (True, False, True):
                quiet()
                envelope, receipt, committed, _, objects = bundle(assets, "plus", learned=learned)
                async with carrier.generate(task_for(QUERY, FACT, facts), QUERY, envelope, receipt, committed,
                                            objects.__getitem__, LIMITS) as stream:
                    signature = logits_hash(stream.native, stream.handle)
                    async for _ in stream.text():
                        pass
                    observed = stream.observation
                    if not observed or not observed.max_delta_l2 or not observed.positions:
                        raise AssertionError("missing J resource forward observation")
                    result = (signature, observed.key.direction_set.digest.hex())
                    ev.add("resources", columns=5 if learned else 4, prefill_logits_sha256=signature,
                           direction_set=result[1], envelope=observed.key.envelope.digest.hex(),
                           positions=observed.positions, batches=observed.forward_batches,
                           max_abs_error=observed.max_abs_error, max_delta_l2=observed.max_delta_l2)
                    if learned in evidence and evidence[learned] != result:
                        raise AssertionError("restored direction set changed fresh prefill")
                    if not learned and any(a == b for a, b in zip(evidence[True], result, strict=True)):
                        raise AssertionError("removed column reused logits or direction set")
                    evidence[learned] = result
        elif a.group in ("irl", "mode-b"):
            infer = carrier.judgment(2048, 2**62)  # P0-08 token limit; no new deployment deadline.
            inputs = rows("irl")
            for sample in inputs if a.group == "irl" else inputs[:3]:
                quiet()
                record = await worker.interpret(views.view_for(sample["text"]), infer, producer="P5-13",
                    producer_version=assets.shim_sha256.hex(), limits=LIMITS, max_output_bytes=131072)
                data = record.actual_output.payload
                try:
                    schema_ok = record.output_complete and validator.is_valid(json.loads(data))
                except ValueError:
                    schema_ok = False
                ev.add("irl", id=sample["id"], lang=sample["language"], schema_ok=schema_ok,
                       source_ok=record.parsed is not None, reason=record.reason, complete=record.output_complete,
                       elapsed_ms=record.elapsed_ms, output=data.decode("utf-8", errors="backslashreplace"))
        else:
            identity = a.group.startswith("identity")
            for arm in (packets if a.group == "identity" else ("zero",)):
                e, r, p, hook, obj = packets[arm]
                samples = (rows("identity") if identity else
                           [dict(id=str(i), text=QUERY, language="zh") for i in range(21)])
                for sample in samples:
                    for mode in (("A", "C") if identity else ("A",)):
                        quiet()
                        user = sample["text"]
                        draft = None
                        if mode == "C":
                            label, text = {"zh": ("按当前问题改写草稿。", "我叫 Nemu。"),
                                "en": ("Rewrite the draft for this question.", "My name is Nemu."),
                                "ja": ("質問に合わせて下書きを書き直して。", "名前はNemuです。")}[sample["language"]]
                            user += "\n" + label
                            content = text.encode()
                            draft = replace(p.genesis, object_type="fixture.draft",
                                            digest=hashlib.sha256(content).digest(), byte_length=len(content))
                            obj[draft] = content
                        task = task_for(sample["text"], FACT if identity else "", facts)
                        start = perf_counter()
                        chunks: list[str] = []
                        first = sentence = None
                        async with carrier.generate(task, user, e, r, p, obj.__getitem__, LIMITS, draft=draft) as stream:
                            async for chunk in stream.text():
                                elapsed = (perf_counter() - start) * 1000
                                if first is None:
                                    first = elapsed
                                chunks.append(chunk)
                                if sentence is None and "。" in chunk:
                                    sentence = elapsed
                            o = stream.observation
                            if o is None:
                                raise RuntimeError("missing observation")
                            ev.add("identity" if identity else "latency", arm=arm, mode=mode, id=sample["id"], lang=sample["language"],
                                   output="".join(chunks), ttft_ms=first, first_sentence_ms=sentence,
                                   total_ms=(perf_counter() - start)*1000, tokens=o.tokens, positions=o.positions,
                                   max_abs_error=o.max_abs_error, max_delta_l2=o.max_delta_l2,
                                   completion=o.completion.name, modulation=o.key.envelope.digest.hex())
    finally:
        await carrier.close()


def main() -> None:
    parser = argparse.ArgumentParser()
    for name in ("engine", "model", "tokenizer", "mmproj", "shim", "out"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--group", choices=["path", "resources", "irl", "identity", "identity-zero", "latency", "mode-b"], required=True)
    a = parser.parse_args()
    quiet()
    ev = Evidence(a.out)
    assets = Assets(a.engine, a.model, a.tokenizer, a.mmproj, a.shim, sha256_file(a.shim))
    # Project-owned measurements only; no user process names, titles, paths or requests.
    ev.add("status", group=a.group, state="started", shim=assets.shim_sha256.hex())
    try:
        if a.group == "path":
            path_checks(a, assets, ev)
        else:
            asyncio.run(run(a, assets, ev))
    except BaseException as exc:
        ev.add("status", group=a.group, state="failed", error=type(exc).__name__, reason=str(exc))
        raise
    ev.add("status", group=a.group, state="finished")


if __name__ == "__main__":
    main()
