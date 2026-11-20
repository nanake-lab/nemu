import json
import unittest
from typing import Any

from plan_speak import FALLBACK, PLAN2_FALLBACK, join_points, verify_points

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


class Plan2Tests(unittest.TestCase):
    def verify(self, kind: str, text: str, mode: str = "loose") -> dict[str, Any]:
        point = {"type": kind, "text": text, "cites": ["n1"] if kind == "past_self" else []}
        return verify_points(
            json.dumps({"points": [point]}), SPEAKERS,
            self_quote=mode, record_texts={"n1": "是不是忘了。"},
        )

    def test_reject_past_now(self) -> None:
        result = self.verify("now", "那天我愣了一下。")
        self.assertEqual(result["checks"][0]["reason"], "now_refers_past")

    def test_keep_now_with_present_marker(self) -> None:
        self.assertEqual(len(self.verify("now", "现在回想那天，还有点闷。")["kept"]), 1)

    def test_strict_accepts_verbatim(self) -> None:
        self.assertEqual(len(self.verify("past_self", "我想：是不是忘了。", "strict")["kept"]), 1)

    def test_strict_rejects_paraphrase(self) -> None:
        self.assertEqual(
            self.verify("past_self", "我想你是不是忘了？", "strict")["checks"][0]["reason"],
            "not_verbatim",
        )

    def test_loose_accepts_same_paraphrase(self) -> None:
        self.assertEqual(len(self.verify("past_self", "我想你是不是忘了？")["kept"]), 1)

    def test_join_points(self) -> None:
        points = [{"text": text} for text in ("你好", "是吗？", "我记得…", "再见")]
        self.assertEqual(join_points(points), "你好。是吗？我记得…再见")

    def test_empty_after_filtering(self) -> None:
        result = self.verify("now", "当时我没说话。")
        self.assertEqual(result["kept"], [])
        self.assertEqual(join_points(result["kept"]), PLAN2_FALLBACK)


if __name__ == "__main__":
    unittest.main()
