"""Numeric IDL rules may only refer to required scalar doubles (P5-34)."""
from __future__ import annotations

import copy
import shutil
import tempfile
import unittest
from pathlib import Path

from generate import HERE, schema
from check_compat import violations


class RetentionSchemaTests(unittest.TestCase):
    def test_rules_and_reserved_inheritance(self) -> None:
        messages = schema()["messages"]
        self.assertEqual(messages["EvidenceAllocationV3"]["rules"], {"positive": ["actual_retention"]})
        self.assertEqual(messages["TendencyV3"]["rules"], {"at_least_abs": {"evidence_absolute": "evidence_a"}})
        self.assertEqual(messages["EvidenceGroupV3"]["reserved"], [7, 8])
        self.assertEqual(messages["RSnapshotV3"]["reserved"], [4, 5])

    def test_numeric_rule_targets_cannot_be_optional_repeated_or_nonnumeric(self) -> None:
        for replacement in ('"projection"', '"tendency_id"', '"absent"'):
            with self.subTest(replacement=replacement), tempfile.TemporaryDirectory() as tmp:
                target = Path(tmp)
                for p in HERE.glob("*.proto"):
                    shutil.copyfile(p, target / p.name)
                path = target / "law.proto"
                path.write_text(path.read_text(encoding="utf-8").replace(
                    '"positive":["actual_retention"]', '"positive":[' + replacement + ']'), encoding="utf-8")
                with self.assertRaises(AssertionError):
                    schema(target)

    def test_repeated_double_cannot_acquire_a_scalar_numeric_rule(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            target = Path(tmp)
            for p in HERE.glob("*.proto"):
                shutil.copyfile(p, target / p.name)
            path = target / "law.proto"
            path.write_text(path.read_text(encoding="utf-8").replace(
                "optional double actual_retention", "repeated double actual_retention"), encoding="utf-8")
            with self.assertRaises(AssertionError):
                schema(target)

    def test_numeric_rules_cannot_be_dropped_after_publication(self) -> None:
        current = schema()
        for message in ("EvidenceAllocationV3", "TendencyV3"):
            changed = copy.deepcopy(current)
            changed["messages"][message]["rules"] = {}
            self.assertIn(f"message {message} rules changed", violations(current, changed))


if __name__ == "__main__":
    unittest.main()
