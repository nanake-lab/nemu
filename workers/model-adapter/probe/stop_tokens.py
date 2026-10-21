"""Use the pinned model generation configuration's complete EOS set."""
from __future__ import annotations


def eos_ids(value: object) -> frozenset[int]:
    if isinstance(value, int) and not isinstance(value, bool):
        return frozenset((value,))
    if isinstance(value, list) and value and all(
        isinstance(item, int) and not isinstance(item, bool) for item in value
    ):
        return frozenset(value)
    raise ValueError("model generation_config must declare integer EOS token(s)")
