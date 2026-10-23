"""The resource projection is part of the authority contract, not a parallel TS schema."""

from __future__ import annotations
import copy
import unittest
from generate import schema
from generate_browser import resource_types
from check_compat import violations


class BrowserSchemaTests(unittest.TestCase):
    def test_resource_fields_are_derived(self) -> None:
        current = schema()
        text = resource_types(current)
        self.assertIn("cooldownS: Record<string, number>", text)
        self.assertIn("autonomousHelperPerDay: number", text)
        changed = copy.deepcopy(current)
        changed["messages"]["ProactivityBudgets"]["fields"][1]["json_name"] = (
            "changedBudget"
        )
        self.assertIn("changedBudget: number", resource_types(changed))
        self.assertNotIn("minIntervalS: number", resource_types(changed))
        self.assertIn(
            "message ProactivityBudgets field 2 changed", violations(current, changed)
        )

    def test_enum_alias_changes_are_not_invisible(self) -> None:
        current = schema()
        changed = copy.deepcopy(current)
        changed["json_enums"]["ProactiveKind"]["PROACTIVE_SPEAK"] = "different"
        self.assertIn(
            "enum ProactiveKind JSON aliases changed", violations(current, changed)
        )

    def test_projection_cannot_silently_disappear(self) -> None:
        current = schema()
        changed = copy.deepcopy(current)
        del changed["messages"]["ProactivityBudgets"]["resource"]
        self.assertIn(
            "message ProactivityBudgets resource projection changed",
            violations(current, changed),
        )


if __name__ == "__main__":
    unittest.main()
