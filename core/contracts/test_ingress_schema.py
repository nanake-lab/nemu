"""P5-35 metadata guards cannot silently revive persistence or reuse fields."""

import copy
import unittest
from check_compat import violations
from generate import schema


class IngressSchemaTests(unittest.TestCase):
    def test_retained_receipt_keeps_every_old_field_or_reserves_it(self) -> None:
        original = schema()
        self.assertEqual(violations(original, original), [])
        broken = copy.deepcopy(original)
        field = next(
            f for f in broken["messages"]["ReceiptV2"]["fields"] if f["number"] == 4
        )
        field["required"] = True
        self.assertTrue(
            any("reuses Receipt.event_id" in v for v in violations(original, broken))
        )

    def test_transient_cannot_be_revived_or_embedded_in_persistent_message(
        self,
    ) -> None:
        original = schema()
        broken = copy.deepcopy(original)
        del broken["messages"]["TerminalAuthenticate"]["transient"]
        self.assertTrue(
            any("persistence classification" in v for v in violations(original, broken))
        )
        broken = copy.deepcopy(original)
        broken["messages"]["TerminalRequest"]["fields"][5]["type"] = (
            "TerminalAuthenticate"
        )
        self.assertTrue(
            any(
                "embeds transient credentials" in v
                for v in violations(original, broken)
            )
        )
