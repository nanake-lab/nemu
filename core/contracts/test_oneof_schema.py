"""Text-level regressions for the ADR 0007 oneof descriptor boundary."""

from __future__ import annotations
import copy
from pathlib import Path
import tempfile
import unittest
from generate import schema
from check_compat import violations

HERE = Path(__file__).resolve().parent


class OneofSchemaTests(unittest.TestCase):
    def parsed(self, before: str, after: str) -> dict[str, object]:
        with tempfile.TemporaryDirectory(prefix="nemu-idl-") as temporary:
            target = Path(temporary)
            for source in HERE.glob("*.proto"):
                text = source.read_text(encoding="utf-8")
                if source.name == "terminal.proto":
                    assert before in text
                    text = text.replace(before, after, 1)
                (target / source.name).write_text(text, encoding="utf-8")
            return schema(target)

    def test_unannotated_field_is_not_silently_ignored(self) -> None:
        for declaration in (
            "optional string sneaky = 99;",
            "string sneaky = 99; // required",
            "map<string, string> sneaky = 99;",
        ):
            with (
                self.subTest(declaration=declaration),
                self.assertRaisesRegex(AssertionError, "unparsed"),
            ):
                self.parsed(
                    "message ReadoutV2 {", "message ReadoutV2 {\n  " + declaration
                )

    def test_reserved_ranges_fail_closed(self) -> None:
        with self.assertRaisesRegex(AssertionError, "unparsed reserved"):
            self.parsed("reserved 2;", "reserved 2 to 9;")

    def test_oneof_members_need_annotations(self) -> None:
        with self.assertRaisesRegex(AssertionError, "unparsed"):
            self.parsed("BodyReadout v1 = 5; // optional", "BodyReadout v1 = 5;")

    def test_oneof_numbers_are_unique_and_ascending(self) -> None:
        for number in (4, 5):
            with self.subTest(number=number), self.assertRaises(AssertionError):
                self.parsed(
                    "BodyReadoutV2 v2 = 6; // optional",
                    f"BodyReadoutV2 v2 = {number}; // optional",
                )

    def test_membership_changes_break_compatibility(self) -> None:
        before = schema()
        after = copy.deepcopy(before)
        after["messages"]["ReadoutV2"]["rules"]["oneofs"]["body"]["fields"].pop()
        self.assertIn("message ReadoutV2 rules changed", violations(before, after))


if __name__ == "__main__":
    unittest.main()
