"""CPU checks for source validation used after constrained model decoding."""
import importlib
import json
import unittest
from types import SimpleNamespace
from unittest.mock import patch

from local_irl import SPIKE, preflight, read_json, source_valid


class SourceValidationTests(unittest.TestCase):
    def test_wire_schema_accepts_a_source_bound_graph(self) -> None:
        enforcer = importlib.import_module("lmformatenforcer")
        parser = enforcer.JsonSchemaParser(read_json(SPIKE / "semantic-candidate.wire.schema.json"))
        value = {"schemaVersion": "p0-08-candidate-v1", "sourceRef": "p0-08-0001", "nodes": []}
        for character in json.dumps(value):
            self.assertIn(character, parser.get_allowed_characters())
            parser = parser.add_character(character)
        self.assertTrue(parser.can_end())

    def test_wire_schema_parses_all_three_node_kinds(self) -> None:
        enforcer = importlib.import_module("lmformatenforcer")
        common = {
            "id": "n1", "sourceRefs": ["p0-08-0001"], "sourceSpan": [0, 4],
            "derivation": "verbatim", "epistemicType": "user_self_report",
            "resolutionState": "resolved", "representationVersion": "p0-08-candidate-v1",
        }
        variants: list[dict[str, object]] = [
            {"kind": "Ref", "mention": "work"},
            {"kind": "Act", "actType": "other", "content": "done"},
            {"kind": "Claim", "predicateLabel": "done", "roles": [], "value": True,
             "polarity": "positive", "modality": "asserted", "time": None},
        ]
        for variant in variants:
            with self.subTest(kind=variant["kind"]):
                parser = enforcer.JsonSchemaParser(
                    read_json(SPIKE / "semantic-candidate.wire.schema.json"),
                )
                value = {"schemaVersion": "p0-08-candidate-v1", "sourceRef": "p0-08-0001",
                         "nodes": [dict(common, **variant)]}
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


class QuietWindowTests(unittest.TestCase):
    def test_known_conflicts_stop_before_gpu(self) -> None:
        for name, command in [
            ("Runner.Listener.exe", "runner"), ("Runner.Worker.exe", "job"),
            ("blender.exe", "blender -b"), ("python.exe", "python asr_probe.py"),
            ("python.exe", "python P4-02/tts_probe.py"),
        ]:
            with self.subTest(name=name, command=command):
                result = SimpleNamespace(stdout=json.dumps([
                    {"Name": name, "ProcessId": 123, "CommandLine": command},
                ]))
                with patch("local_irl.subprocess.run", return_value=result):
                    with self.assertRaisesRegex(RuntimeError, "Quiet window conflict"):
                        preflight()

    def test_unknown_inventory_or_host_command_stops(self) -> None:
        for inventory in [None, [], [
            {"Name": "python.exe", "ProcessId": 123, "CommandLine": None},
        ]]:
            with self.subTest(inventory=inventory):
                with patch("local_irl.subprocess.run", return_value=SimpleNamespace(
                    stdout=json.dumps(inventory),
                )):
                    with self.assertRaises(RuntimeError):
                        preflight()

    def test_known_safe_measurement_host_is_allowed(self) -> None:
        result = SimpleNamespace(stdout=json.dumps([
            {"Name": "python.exe", "ProcessId": 123,
             "CommandLine": "python local_irl.py --output local-irl-evidence"},
        ]))
        with patch("local_irl.subprocess.run", return_value=result):
            self.assertEqual(preflight(), [])


if __name__ == "__main__":
    unittest.main()
