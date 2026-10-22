"""CPU regression for the observed literal-control bug in pinned LMFE."""
import importlib
import json
from typing import Protocol, cast
import unittest

from decoder_adapter import harden_tokenizer_data


class AllowedTokens(Protocol):
    allowed_tokens: list[int]


class Enforcer(Protocol):
    def get_allowed_tokens(self, tokens: list[int]) -> AllowedTokens: ...


class DecoderAdapterTests(unittest.TestCase):
    def build(self, harden: bool) -> tuple[Enforcer, dict[str, int], int]:
        api = importlib.import_module("lmformatenforcer")
        chars = [chr(index) for index in range(128)] + ["你"]
        ids = {char: index for index, char in enumerate(chars)}
        inverse = {token_id: char for char, token_id in ids.items()}
        eos = len(chars)
        data = api.TokenEnforcerTokenizerData(
            [(token_id, char, True) for char, token_id in ids.items()],
            lambda tokens: "".join(inverse[token] for token in tokens),
            eos, False, eos + 1,
        )
        if harden:
            data, _ = harden_tokenizer_data(data)
        schema = {"type": "object", "properties": {"x": {"type": "string"}},
                  "required": ["x"], "additionalProperties": False}
        config = api.CharacterLevelParserConfig(alphabet=data.tokenizer_alphabet)
        parser = api.JsonSchemaParser(schema, config=config)
        return cast(Enforcer, api.TokenEnforcer(data, parser)), ids, eos

    def allowed(self, enforcer: Enforcer, ids: dict[str, int], text: str) -> list[int]:
        tokens: list[int] = []
        for char in text:
            self.assertIn(ids[char], enforcer.get_allowed_tokens(tokens).allowed_tokens)
            tokens.append(ids[char])
        return enforcer.get_allowed_tokens(tokens).allowed_tokens

    def test_reproduces_pinned_control_cache_bug(self) -> None:
        enforcer, ids, _ = self.build(False)
        allowed = self.allowed(enforcer, ids, '{"x":"A')
        self.assertIn(ids[chr(9)], allowed)

    def test_rejects_all_literal_string_controls(self) -> None:
        enforcer, ids, eos = self.build(True)
        allowed = self.allowed(enforcer, ids, '{"x":"A')
        for codepoint in range(32):
            self.assertNotIn(ids[chr(codepoint)], allowed)
        self.assertNotIn(eos, allowed)
        self.assertIn(ids['"'], allowed)
        self.assertIn(ids['你'], allowed)

    def test_keeps_escaped_controls_unicode_and_structural_whitespace(self) -> None:
        enforcer, ids, eos = self.build(True)
        document = '{\n"x":\t"你\\tA"\n}'
        allowed = self.allowed(enforcer, ids, document)
        self.assertIn(eos, allowed)
        self.assertEqual(json.loads(document), {"x": "你\tA"})


if __name__ == "__main__":
    unittest.main()
