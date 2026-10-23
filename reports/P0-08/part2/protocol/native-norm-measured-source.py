"""Optional native RMSNorm for the locked carrier families, never the TTS model.

Keep the original low-precision rounding before learned-weight multiplication.
The reduction kernel can differ numerically; path probes must be rerun.
"""
from __future__ import annotations

from typing import cast

import torch
from torch import nn
from torch.nn import functional as functional


class NativeRMSNorm(nn.Module):
    def __init__(self, original: nn.Module) -> None:
        super().__init__()
        self.weight = cast(nn.Parameter, original.weight)
        self.epsilon = float(cast(float, original.variance_epsilon))

    def forward(self, hidden_states: torch.Tensor) -> torch.Tensor:
        normalized = functional.rms_norm(
            hidden_states, (hidden_states.shape[-1],), weight=None, eps=self.epsilon,
        )
        return self.weight * normalized


def replace_rms_norms(model: nn.Module) -> int:
    """Explicit replacement, no forward hooks and no changes to decoder order."""
    count = 0
    for name, child in list(model.named_children()):
        if type(child).__name__ in {"Qwen3RMSNorm", "Phi3RMSNorm"}:
            setattr(model, name, NativeRMSNorm(child))
            count += 1
        else:
            count += replace_rms_norms(child)
    return count
