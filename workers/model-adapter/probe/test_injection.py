"""CPU-only mechanism tests, not pretrained-model feasibility evidence."""

from dataclasses import replace

import pytest
import torch
from torch import nn

from injection import CacheKey, InjectedDecoder, RequestCache, modulation_digest


class Attention(nn.Module):
    def forward(self, hidden_states: torch.Tensor, **kwargs: object) -> tuple[torch.Tensor, None]:
        return hidden_states, None


class Zero(nn.Module):
    def forward(self, hidden_states: torch.Tensor) -> torch.Tensor:
        return torch.zeros_like(hidden_states)


class Layer(nn.Module):
    def __init__(self) -> None:
        super().__init__()
        self.attention_type = "full_attention"
        self.input_layernorm = nn.Identity()
        self.self_attn = Attention()
        self.post_attention_layernorm = nn.Identity()
        self.mlp = Zero()
        self.resid_attn_dropout = nn.Identity()
        self.resid_mlp_dropout = nn.Identity()


@pytest.mark.parametrize("family", ["qwen3", "phi3"])
def test_zero_mask_and_real_nonzero_difference(family: str) -> None:
    wrapper = InjectedDecoder(Layer(), torch.eye(4)[:, :2], beta=1.0, family=family)
    x = torch.ones(1, 3, 4)
    wrapper.begin((0.0, 0.0), observe=True)
    assert torch.equal(wrapper(x), 2 * x)
    wrapper.end()
    wrapper.begin((0.5, 0.0), observe=True)
    actual = wrapper(x)
    expected = 2 * x
    expected[:, -1, 0] += 0.5
    assert torch.equal(actual, expected)
    assert wrapper.observations[0]["max_abs_error"] == 0
    assert wrapper.observations[0]["observed_l2"] == 0.5
    token = torch.ones(1, 1, 4)
    assert wrapper(token)[0, 0, 0] == 2.5
    assert wrapper.observations[1]["phase"] == "decode"
    with pytest.raises(ValueError, match="single-token"):
        wrapper(x)
    with pytest.raises(ValueError, match="end the current"):
        wrapper.begin((0.0, 0.0))
    wrapper.end()
    with pytest.raises(ValueError, match="active bounded"):
        wrapper(token)


def test_invalid_modulation_and_matrix_are_rejected() -> None:
    with pytest.raises(ValueError, match="operator norm"):
        InjectedDecoder(Layer(), torch.eye(4) * 2, beta=1.0)
    wrapper = InjectedDecoder(Layer(), torch.eye(4)[:, :2], beta=1.0)
    for values in [(float("nan"), 0.0), (float("inf"), 0.0), (2.0, 0.0), (0.0,)]:
        with pytest.raises(ValueError):
            wrapper.begin(values)


def test_each_cache_key_component_invalidates_and_requests_never_reuse() -> None:
    key = CacheKey(
        "model", "tokenizer", "prefix", "carrier", modulation_digest((0.0,)), "mask", "numerical"
    )
    cache = RequestCache()
    for field in key.__dataclass_fields__:
        cache.begin(key)
        marker = object()
        cache.put(key, marker)
        assert cache.get(key) is marker
        changed = replace(key, **{field: "changed"})
        assert cache.get(changed) is None
        with pytest.raises(ValueError, match="inactive"):
            cache.put(key, marker)
    cache.begin(key)
    cache.put(key, object())
    cache.begin(key)
    assert cache.get(key) is None
