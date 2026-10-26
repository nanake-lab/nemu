"""Information-preservation checks against the unchanged original schema."""
from __future__ import annotations

from copy import deepcopy
import json
from pathlib import Path
import unittest
from typing import Any

from schema_validation import validator
from compact_codec import ENUMS, VERSION, compact_schema, expand, shrink

ROOT = Path(__file__).resolve().parents[2]


class CompactTests(unittest.TestCase):
    def setUp(self) -> None:
        original = json.loads((ROOT / "spikes/codex-app-server/semantic-candidate.schema.json").read_text(encoding="utf-8"))
        self.full = validator(original)
        self.compact = validator(compact_schema())
        metadata = {"id": "ref1", "sourceRefs": ["p0-08-0001", "external:quoted-source"],
                    "sourceSpan": [0, 3], "derivation": "verbatim", "epistemicType": "user_assertion",
                    "resolutionState": "resolved", "representationVersion": VERSION}
        self.value: dict[str, Any] = {"schemaVersion": VERSION, "sourceRef": "p0-08-0001", "nodes": [
            metadata | {"kind": "Ref", "mention": '猫\n"猫"'},
            metadata | {"id": "claim1", "kind": "Claim", "predicateLabel": "likes",
                        "roles": [{"role": "subject", "value": "ref1"}], "value": None,
                        "polarity": "positive", "modality": "asserted", "time": None},
            metadata | {"id": "act1", "kind": "Act", "actType": "request", "content": "请保留全部语义。"},
        ]}

    def test_roundtrip_all_node_kinds_and_external_refs(self) -> None:
        self.full.validate(self.value)
        compressed = shrink(self.value)
        self.compact.validate(compressed)
        restored = expand(compressed)
        self.full.validate(restored)
        self.assertEqual(restored, self.value)

    def test_enum_and_value_variants_preserve_information(self) -> None:
        for field, options in ENUMS.items():
            if field == "kind":
                continue
            node_index = 2 if field == "actType" else 1
            for option in options:
                value = deepcopy(self.value)
                value["nodes"][node_index][field] = option
                self.full.validate(value)
                self.compact.validate(shrink(value))
                self.assertEqual(expand(shrink(value)), value)
        for primitive in [None, True, False, 0, -1.25, "Unicode 日文/中文"]:
            value = deepcopy(self.value)
            value["nodes"][1]["value"] = primitive
            self.assertEqual(expand(shrink(value)), value)

    def test_unknown_fields_and_bad_codes_rejected(self) -> None:
        value = deepcopy(self.value)
        value["nodes"][0]["hidden"] = 1
        with self.assertRaises(ValueError):
            shrink(value)
        compressed = shrink(self.value)
        compressed["n"][0]["k"] = 3
        with self.assertRaises(ValueError):
            expand(compressed)


if __name__ == "__main__":
    unittest.main()
