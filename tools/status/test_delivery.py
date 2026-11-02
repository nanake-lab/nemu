"""BOARD-PARTIAL-01: report evidence, resolution and main-only reads."""

import json
import unittest
from unittest.mock import Mock

from delivery import delivery_waiting, load_deliveries


class DeliveryTests(unittest.TestCase):
    def test_current_partial_and_named_successor_request(self) -> None:
        report = (
            "# 报告\n## 当前状态\n部分交付；等待 P5-06 和 "
            "[正式接线](../../tasks/requests/P5-24-inputs.md)。\n"
            "## 历史\n等待 P5-99\n"
        )
        self.assertEqual(delivery_waiting("P5-24", report, ""), ["正式接线", "P5-06"])

    def test_complete_and_resolved_blockers_do_not_revive_history(self) -> None:
        report = "## 当前状态\n完成。\n## 历史\n部分交付，等待 P5-06。"
        for blocked in (
            "", "# 阻塞\n- [x] P5-06 接线\n",
            "# 阻塞\n## 已解除\n等待 P5-06。",
            "# 阻塞\n## 当前状态\n全部已解除。\n## 历史\n- [ ] P5-06",
        ):
            with self.subTest(blocked=blocked):
                self.assertIsNone(delivery_waiting("P5-24", report, blocked))

    def test_open_blocker_wins_over_complete_report(self) -> None:
        for blocked in (
            "# 阻塞\n- [x] P5-01 完成\n- [ ] 等待 P5-06 接线",
            "# 阻塞\n## 已解除\nP5-01 接线\n## 尚未解除\n等待 P5-06。",
            "# 阻塞\n| 阻塞 | 接续 |\n| --- | --- |\n| 接线 | P5-06 |",
            "# 阻塞\n等待 P5-06。\n\n已试过：核对 P5-23 API。",
        ):
            with self.subTest(blocked=blocked):
                self.assertEqual(delivery_waiting("P5-24", "## 当前状态\n完成。", blocked),
                                 ["P5-06"])

    def test_unnamed_successor_points_to_evidence(self) -> None:
        self.assertEqual(delivery_waiting("P5-24", "## 当前状态\n部分完成。", ""),
                         ["见 reports/P5-24/REPORT.md"])

    def test_main_tree_pins_reads_and_missing_files_mean_no_blocker(self) -> None:
        def raw(args: list[str]) -> str:
            if args[-1].endswith("git/trees/main?recursive=1"):
                return json.dumps({"tree": [
                    {"path": "reports/P5-24/REPORT.md", "sha": "report-blob"},
                    {"path": "reports/P5-24/BLOCKED.md", "sha": "blocked-blob"},
                ]})
            if args[-1].endswith("git/blobs/report-blob"):
                return "## 当前状态\n部分交付。"
            if args[-1].endswith("git/blobs/blocked-blob"):
                return "- [ ] 等待 P5-06"
            self.fail(f"Unexpected request: {args}")

        self.assertEqual(load_deliveries("example/test", {"P5-24", "P5-25"}, raw),
                         {"P5-24": ["P5-06"]})

    def test_unreadable_or_incomplete_main_is_not_completion(self) -> None:
        for raw in (
            Mock(side_effect=RuntimeError("offline")),
            Mock(return_value='{"truncated": true}'),
        ):
            with self.subTest(raw=raw), self.assertRaises(RuntimeError):
                load_deliveries("example/test", {"P5-24"}, raw)


if __name__ == "__main__":
    unittest.main()
