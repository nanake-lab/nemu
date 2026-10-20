"""Shared offline Instruct probe runtime; no dialogue persistence or core storage."""
from __future__ import annotations

from dataclasses import asdict, dataclass
import hashlib
import importlib
import json
import math
from pathlib import Path
import re
import time
from typing import Protocol, cast

import torch
from torch import nn

from direction_probe import NEUTRAL_PROMPTS, PAIRS
from injection import InjectedDecoder
from native_norm import replace_rms_norms
from path_probe import forward
from runtime_guard import require_runner_offline
from stop_tokens import eos_ids

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
MODEL = "Qwen/Qwen3-4B-Instruct-2507"
REVISION = "cdbee75f17c01a7cc42f958dc650907174af0554"
PLAN = HERE / "instruct-dose-plan-v1.json"


class Tokenizer(Protocol):
    def apply_chat_template(self, conversation: list[dict[str, str]], *, tokenize: bool,
                            add_generation_prompt: bool, enable_thinking: bool) -> str: ...
    def __call__(self, text: str, *, return_tensors: str) -> dict[str, torch.Tensor]: ...
    def decode(self, ids: list[int], *, skip_special_tokens: bool) -> str: ...


@dataclass
class Generation:
    output: str
    token_ids: list[int]
    stop: str
    elapsed_s: float
    first_sentence_ms: float | None

    def to_dict(self) -> dict[str, object]:
        return asdict(self)


class InstructRuntime:
    def __init__(self) -> None:
        require_runner_offline()
        torch.set_num_threads(4)
        torch.manual_seed(731)
        torch.backends.cuda.matmul.allow_tf32 = False
        torch.backends.cudnn.allow_tf32 = False
        manifest = json.loads((HERE / "manifest.json").read_text(encoding="utf-8"))
        entry = next(m for m in manifest["models"] if m["id"] == MODEL)
        if entry["revision"] != REVISION:
            raise ValueError("Instruct revision mismatch")
        local = HERE / ".cache/models" / MODEL.replace("/", "--") / REVISION
        api = importlib.import_module("transformers")
        self.model = cast(nn.Module, api.AutoModelForCausalLM.from_pretrained(
            str(local), local_files_only=True, trust_remote_code=False, dtype=torch.bfloat16,
            device_map={"": "cuda:0"}, attn_implementation="eager",
            quantization_config=api.BitsAndBytesConfig(
                load_in_4bit=True, bnb_4bit_quant_type="nf4",
                bnb_4bit_compute_dtype=torch.bfloat16, bnb_4bit_use_double_quant=False,
            ),
        )).eval().requires_grad_(False)
        self.tokenizer = cast(Tokenizer, api.AutoTokenizer.from_pretrained(
            str(local), local_files_only=True,
        ))
        self.native_rms_count = replace_rms_norms(self.model)
        self.layers = cast(nn.ModuleList, cast(nn.Module, self.model.model).layers)
        config = json.loads((local / "config.json").read_text(encoding="utf-8"))
        self.context_limit = int(config["max_position_embeddings"])
        self.index = int(config["num_hidden_layers"]) // 2 - 1
        if self.index != 17:
            raise ValueError("Existing hook must remain zero-based layer 17")
        self.original = self.layers[self.index]
        self.width = int(config["hidden_size"])
        generation = json.loads((local / "generation_config.json").read_text(encoding="utf-8"))
        self.stopping = eos_ids(generation["eos_token_id"])
        plan = json.loads(PLAN.read_text(encoding="utf-8"))
        source = ROOT / plan["direction_source"]
        archived = json.loads(source.read_text(encoding="utf-8"))
        if (archived["model"]["id"] != MODEL or archived["model"]["revision"] != REVISION
                or archived["calibration_pairs"] != [list(pair) for pair in PAIRS]
                or archived["generation_prompts"] != NEUTRAL_PROMPTS
                or archived["layer_index_zero_based"] != self.index):
            raise ValueError("Archived direction binding mismatch")
        self.u = torch.tensor(archived["u"], dtype=torch.float32)
        self.u_sha256 = hashlib.sha256(self.u.numpy().tobytes()).hexdigest()
        if self.u_sha256 != archived["u_sha256"] or self.u.shape != (self.width, 8):
            raise ValueError("Archived U digest or shape mismatch")
        self.direction_norm = float(torch.linalg.vector_norm(self.u[:, 0].double()))
        self.wrapper: InjectedDecoder | None = None

    def install(self, beta: float) -> InjectedDecoder:
        self.wrapper = InjectedDecoder(self.original, self.u.to("cuda:0"), beta=beta)
        self.layers[self.index] = self.wrapper
        return self.wrapper

    def generate(self, messages: list[dict[str, str]], coefficient: float, seed: int,
                 *, observe: bool = False) -> Generation:
        require_runner_offline()
        wrapper = self.wrapper
        if wrapper is None:
            raise RuntimeError("Install the fixed wrapper before generating")
        text = self.tokenizer.apply_chat_template(messages, tokenize=False,
                                                 add_generation_prompt=True, enable_thinking=False)
        ids = self.tokenizer(text, return_tensors="pt")["input_ids"]
        if ids.shape[1] + 48 > self.context_limit:
            raise ValueError("Context exceeds the model limit; use /reset")
        ids = ids.to("cuda:0")
        torch.manual_seed(seed)
        tokens: list[int] = []
        kv: object = None
        current = ids
        first_sentence: float | None = None
        stop = "token_limit"
        wrapper.begin((coefficient,) + (0.0,) * 7, observe=observe)
        torch.cuda.synchronize()
        started = time.perf_counter()
        try:
            with torch.inference_mode():
                for _ in range(48):
                    out = forward(self.model, current, kv)
                    kv = out.past_key_values
                    probabilities = torch.softmax(out.logits[:, -1, :].float() / 0.7, dim=-1)
                    current = torch.multinomial(probabilities, 1)
                    token = int(current.item())
                    tokens.append(token)
                    torch.cuda.synchronize()
                    elapsed = time.perf_counter() - started
                    output = self.tokenizer.decode(tokens, skip_special_tokens=True)
                    if first_sentence is None and re.search(r"[。！？!?\n]|(?<!\d)\.(?:\s|$)", output):
                        first_sentence = elapsed * 1000
                    if token in self.stopping:
                        stop = "eos"
                        break
                    if elapsed >= 25:
                        stop = "time_limit"
                        break
            return Generation(self.tokenizer.decode(tokens, skip_special_tokens=True), tokens,
                              stop, time.perf_counter() - started, first_sentence)
        finally:
            wrapper.end()


def coefficient_for_ratio(ratio: float, reference_norm: float, direction_norm: float) -> float:
    if not all(math.isfinite(x) for x in (ratio, reference_norm, direction_norm)):
        raise ValueError("Finite ratio and norms required")
    if reference_norm <= 0 or direction_norm <= 0:
        raise ValueError("Positive norms required")
    return ratio * reference_norm / direction_norm
