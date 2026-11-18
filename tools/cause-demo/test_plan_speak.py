import json
import unittest
from typing import Any

from plan_speak import FALLBACK, verify_points

SPEAKERS = {"e4": "网友阿杰", "n1": "Nemu（独处时的念头）"}


class VerifyPointsTests(unittest.TestCase):
    def check_rejected(self, point: Any, reason: str) -> None:
        result = verify_points(json.dumps({"points": [point]}), SPEAKERS)
        self.assertEqual(result["checks"], [{"point": point, "kept": False, "reason": reason}])
        self.assertEqual(result["kept"], [{"type": "other", "text": FALLBACK, "cites": []}])

    def test_invalid_json(self) -> None:
        with self.assertRaises(json.JSONDecodeError):
            verify_points("{", SPEAKERS)

    def test_invalid_shape(self) -> None:
        with self.assertRaises(TypeError):
            verify_points('{"points": "not an array"}', SPEAKERS)

    def test_invalid_point(self) -> None:
        self.check_rejected("text", "invalid_point")

    def test_unknown_type(self) -> None:
        self.check_rejected({"type": "guess", "text": "要点"}, "unknown_type")

    def test_invalid_text(self) -> None:
        self.check_rejected({"type": "other", "text": " "}, "invalid_text")

    def test_invalid_cites(self) -> None:
        self.check_rejected({"type": "past_event", "text": "要点", "cites": [42]}, "invalid_cites")

    def test_empty_cites(self) -> None:
        self.check_rejected({"type": "past_event", "text": "要点", "cites": []}, "empty_cites")

    def test_undelivered_cite(self) -> None:
        self.check_rejected(
            {"type": "past_self", "text": "要点", "cites": ["n1", "n2"]}, "undelivered_cite"
        )

    def test_no_nemu_cite(self) -> None:
        self.check_rejected({"type": "past_self", "text": "要点", "cites": ["e4"]}, "no_nemu_cite")

    def test_valid_points_keep_order(self) -> None:
        points = [
            {"type": "past_event", "text": "事件", "cites": ["e4"]},
            {"type": "past_self", "text": "念头", "cites": ["e4", "n1"]},
            {"type": "now", "text": "现在", "cites": []},
            {"type": "other", "text": "问题", "cites": []},
        ]
        result = verify_points(json.dumps({"points": points}), SPEAKERS)
        self.assertEqual(result["kept"], points)
        self.assertTrue(all(check["kept"] for check in result["checks"]))


if __name__ == "__main__":
    unittest.main()
