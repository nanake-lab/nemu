"""CPU checks for source validation used after constrained model decoding."""
import importlib
import json
import unittest

from local_irl import SPIKE, read_json, source_valid


class SourceValidationTests(unittest.TestCase):
    def test_wire_schema_accepts_a_source_bound_graph(self) -> None:
        enforcer = importlib.import_module("lmformatenforcer")
        parser = enforcer.JsonSchemaParser(read_json(SPIKE / "semantic-candidate.wire.schema.json"))
        value = {"schemaVersion": "p0-08-candidate-v1", "sourceRef": "p0-08-0001", "nodes": []}
        for character in json.dumps(value):
            self.assertIn(character, parser.get_allowed_characters())
            parser = parser.add_character(character)
        self.assertTrue(parser.can_end())

    def test_unicode_source_span(self) -> None:
        sample: dict[str, object] = {"sourceRef": "p0-08-0001", "text": "😀你好"}
        node: dict[str, object] = {
            "id": "n1", "sourceRefs": ["p0-08-0001"], "sourceSpan": [0, 3],
        }
        value = {"sourceRef": "p0-08-0001", "nodes": [node]}
        self.assertTrue(source_valid(value, sample))
        node["sourceSpan"] = [0, 4]
        self.assertFalse(source_valid(value, sample))

    def test_missing_source_and_duplicate_identity(self) -> None:
        sample: dict[str, object] = {"sourceRef": "p0-08-0001", "text": "你好"}
        node: dict[str, object] = {
            "id": "n1", "sourceRefs": ["other"], "sourceSpan": [0, 2],
        }
        value = {"sourceRef": "p0-08-0001", "nodes": [node]}
        self.assertFalse(source_valid(value, sample))
        node["sourceRefs"] = ["p0-08-0001"]
        value["nodes"] = [node, dict(node)]
        self.assertFalse(source_valid(value, sample))


if __name__ == "__main__":
    unittest.main()
