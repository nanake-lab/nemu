"""Pinned LMFE adapter: exclude literal JSON string controls, retain structural whitespace."""
from __future__ import annotations

import importlib
from importlib.metadata import version
from typing import Protocol, cast


class FreeTextCache(Protocol):
    def add_token(self, text: str, token_id: int) -> None: ...
    def freeze(self) -> None: ...


class TokenTree(Protocol):
    tokens_to_strs: dict[int, str]
    json_freetext_tokens: FreeTextCache


class TokenizerData(Protocol):
    tokenizer_alphabet: str
    tokenizer_tree: TokenTree
    use_bitmask: bool
    vocab_size: int


def harden_tokenizer_data(raw: object) -> tuple[TokenizerData, dict[str, object]]:
    """Correct two control-character allowlists without changing the wire schema."""
    actual_version = version("lm-format-enforcer")
    if actual_version != "0.11.3":
        raise RuntimeError("The decoder adapter requires review for this LMFE version")
    data = cast(TokenizerData, raw)
    alphabet = data.tokenizer_alphabet
    data.tokenizer_alphabet = "".join(char for char in alphabet if ord(char) >= 0x20)
    api = importlib.import_module("lmformatenforcer.tokenizerprefixtree")
    cache = cast(FreeTextCache, api.JsonFreetextTokenCache(data.use_bitmask, data.vocab_size))
    excluded = 0
    for token_id, text in data.tokenizer_tree.tokens_to_strs.items():
        if any(ord(char) < 0x20 for char in text):
            excluded += 1
        else:
            cache.add_token(text, token_id)
    cache.freeze()
    data.tokenizer_tree.json_freetext_tokens = cache
    return data, {
        "version": "strict-json-controls-v2", "lmFormatEnforcer": actual_version,
        "excludedAlphabetControls": len(alphabet) - len(data.tokenizer_alphabet),
        "excludedFreeStringTokens": excluded,
        "rule": "No literal U+0000..U+001F inside strings; escaped controls and structural whitespace remain legal",
        "dependencyFilesModified": False,
    }
