"""Explicit Qwen3/Phi3 residual wrapper for P0-07; not a production carrier.

Forward order adapted from Transformers 4.57.3 Qwen3DecoderLayer:
Copyright 2025 The Qwen team, Alibaba Group and the HuggingFace Inc. team.
Phi3 forward order: Copyright 2024 Microsoft and the HuggingFace Inc. team.
Licensed under Apache-2.0: https://www.apache.org/licenses/LICENSE-2.0
Modification: explicit low-rank addition after attention residual, before MLP norm.
"""

from __future__ import annotations

from dataclasses import dataclass
import hashlib
import json
import math
from typing import cast

import torch
from torch import nn


@dataclass(frozen=True)
class CacheKey:
    model: str
    tokenizer: str
    prompt_prefix: str
    carrier: str
    modulation: str
    mask: str
    numerical: str


def modulation_digest(values: tuple[float, ...]) -> str:
    if not all(math.isfinite(v) for v in values):
        raise ValueError("modulation must be finite")
    return hashlib.sha256(json.dumps(values, allow_nan=False).encode("ascii")).hexdigest()


class RequestCache:
    """No cross-request KV reuse. A changed key discards KV before a forward."""

    def __init__(self) -> None:
        self.key: CacheKey | None = None
        self.value: object = None

    def begin(self, key: CacheKey) -> None:
        self.key = key
        self.value = None

    def get(self, key: CacheKey) -> object:
        if self.key != key:
            self.begin(key)
        return self.value

    def put(self, key: CacheKey, value: object) -> None:
        if key != self.key:
            raise ValueError("cannot store KV under an inactive modulation key")
        self.value = value


class InjectedDecoder(nn.Module):
    """One fixed layer; batch size 1; last prefill token and each decode token.

    The probe runs a whole unpadded prompt, then one token per decode forward.
    Chunked prefill and padded batches are deliberately rejected by the caller.
    The recorded error is raw evidence, not an assigned acceptance tolerance.
    """

    def __init__(
        self, original: nn.Module, u: torch.Tensor, beta: float, family: str = "qwen3",
        cache_delta: bool = False,
    ) -> None:
        super().__init__()
        if u.ndim != 2 or not torch.isfinite(u).all() or not math.isfinite(beta) or beta <= 0:
            raise ValueError("finite matrix U and positive finite beta required")
        norm = float(torch.linalg.matrix_norm(u.double().cpu(), ord=2).item())
        if norm > 1.0:
            raise ValueError("U operator norm exceeds 1")
        self.original = original
        if family not in {"qwen3", "phi3"}:
            raise ValueError("unsupported decoder family")
        self.family = family
        if family == "qwen3":
            self.attention_type = cast(str, original.attention_type)
        self.register_buffer("u", u.detach().clone())
        self.beta = beta
        self.cache_delta = cache_delta
        self.request_delta: torch.Tensor | None = None
        self.coefficients: tuple[float, ...] | None = None
        self.observe = False
        self.observations: list[dict[str, object]] = []
        self.calls = 0

    def begin(self, values: tuple[float, ...], *, observe: bool = False) -> None:
        if self.coefficients is not None:
            raise ValueError("end the current request before changing modulation")
        u = cast(torch.Tensor, self.u)
        modulation_digest(values)
        if len(values) != u.shape[1] or math.hypot(*values) > self.beta:
            raise ValueError("rank or modulation budget violation")
        self.coefficients = values
        self.observe = observe
        self.observations = []
        self.calls = 0
        self.request_delta = None

    def end(self) -> None:
        self.coefficients = None
        self.request_delta = None

    def forward(self, hidden_states: torch.Tensor, **kwargs: object) -> torch.Tensor:
        if self.coefficients is None or hidden_states.shape[0] != 1:
            raise ValueError("active bounded request and batch size 1 required")
        if self.calls > 0 and hidden_states.shape[1] != 1:
            raise ValueError("after full prefill, only single-token decode is supported")
        layer = self.original
        residual = hidden_states
        normed = cast(nn.Module, layer.input_layernorm)(hidden_states)
        attention, _ = cast(nn.Module, layer.self_attn)(hidden_states=normed, **kwargs)
        if self.family == "phi3":
            attention = cast(nn.Module, layer.resid_attn_dropout)(attention)
        hidden_states = residual + attention
        u = cast(torch.Tensor, self.u)
        if self.cache_delta:
            if self.request_delta is None:
                # First forward stays inside the measured request. Nothing is
                # moved before the timer; only repeated decode work is removed.
                self.request_delta = u @ torch.tensor(
                    self.coefficients, dtype=u.dtype, device=u.device,
                )
            delta = self.request_delta.to(hidden_states.dtype)
        else:
            a = torch.tensor(self.coefficients, dtype=u.dtype, device=u.device)
            delta = (u @ a).to(hidden_states.dtype)
        before = hidden_states[:, -1, :]
        after = before + delta
        if self.observe:
            observed = after.double() - before.double()
            expected = delta.double()
            self.observations.append(
                {
                    "phase": "prefill" if self.calls == 0 else "decode",
                    "sequence_length": hidden_states.shape[1],
                    "injected_position": hidden_states.shape[1] - 1,
                    "before": before.detach().cpu().tolist(),
                    "after": after.detach().cpu().tolist(),
                    "delta": expected.detach().cpu().tolist(),
                    "max_abs_error": float((observed - expected).abs().max().item()),
                    "observed_l2": float(torch.linalg.vector_norm(observed).item()),
                }
            )
        hidden_states = hidden_states.clone()
        hidden_states[:, -1, :] = after
        self.calls += 1
        residual = hidden_states
        hidden_states = cast(nn.Module, layer.post_attention_layernorm)(hidden_states)
        hidden_states = cast(nn.Module, layer.mlp)(hidden_states)
        if self.family == "phi3":
            hidden_states = cast(nn.Module, layer.resid_mlp_dropout)(hidden_states)
        return residual + hidden_states
