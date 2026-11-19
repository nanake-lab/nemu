from __future__ import annotations

import json
from pathlib import Path
from typing import Any
import unittest

from think import PROTOCOL, THOUGHT_SPEAKER, append_thought, projection_payload, render_prompt


def thoughts_scene() -> dict[str, Any]:
    scene: dict[str, Any] = json.loads(
        (Path(__file__).parent / "scenarios" / "thoughts.json").read_text(encoding="utf-8")
    )
    return scene


class ThinkTests(unittest.TestCase):
    def test_projection_filters_future_records_slots_and_relation_endpoints(self) -> None:
        scenario = thoughts_scene()
        at = "2026-11-01 22:00"
        payload = projection_payload(scenario, at)
        *history, trigger = payload["records"]
        self.assertEqual([r["id"] for r in history], ["e1", "n1"])
        self.assertEqual([(s["basis"], s["sources"]) for s in payload["slots"]], [
            ("s_missed", ["e1", "n1"]),
        ])
        self.assertEqual(payload["relations"], [])
        self.assertEqual(trigger, {
            "id": payload["latest"], "h_pos": 71, "speaker": "（时间）",
            "time": at, "text": "现在是 2026-11-01 22:00，你一个人。",
        })
        self.assertEqual((payload["fact_mode"], payload["show_ids"]), ("per_event", False))
        self.assertEqual(scenario, thoughts_scene())

    def test_projection_includes_exact_cutoff_and_visible_relations(self) -> None:
        payload = projection_payload(thoughts_scene(), "2026-11-01 23:40")
        self.assertEqual([r["id"] for r in payload["records"][:-1]], ["e1", "n1", "n2", "e2"])
        self.assertEqual(payload["relations"], [{"from": "e2", "to": "e1", "kind": "body_source"}])

    def test_append_uses_first_accepted_pack_and_preserves_full_scene(self) -> None:
        scenario = thoughts_scene()
        at = "2026-11-05 15:25"
        content = "我不喜欢阿杰那句话。"
        binding = {"packs": [
            {"basis": "s_missed", "fate": "excluded"},
            {"basis": "s_insult", "fate": "accepted"},
            {"basis": "s_word", "fate": "accepted"},
        ]}
        appended = append_thought(scenario, binding, at, content)
        self.assertEqual(appended["records"][:-1], scenario["records"])
        thought = appended["records"][-1]
        self.assertEqual(thought, {
            "id": thought["id"], "h_pos": 71, "speaker": THOUGHT_SPEAKER,
            "time": at, "text": content,
        })
        for before, after in zip(scenario["slots"], appended["slots"], strict=True):
            extra = [thought["id"]] if before["basis"] == "s_insult" else []
            self.assertEqual(after, {**before, "sources": [*before["sources"], *extra]})
        self.assertEqual(appended["relations"], scenario["relations"])
        self.assertEqual(scenario, thoughts_scene())

    def test_append_rejects_projection_without_accepted_pack(self) -> None:
        with self.assertRaisesRegex(ValueError, "没有接纳"):
            append_thought(thoughts_scene(), {"packs": []}, "2026-11-05 15:25", "念头")

    def test_prompt_preserves_e_and_o_in_qwen_non_thinking_template(self) -> None:
        e = "身份\n[当时] 原话。"
        o = "[现在] （时间）：你一个人。"
        messages = [
            {"role": "system", "content": "聊天 P"},
            {"role": "system", "content": e},
            {"role": "user", "content": o},
        ]
        self.assertEqual(
            render_prompt(messages),
            f"<|im_start|>system\n{PROTOCOL}\n\n{e}<|im_end|>\n"
            f"<|im_start|>user\n{o}<|im_end|>\n"
            "<|im_start|>assistant\n<think>\n\n</think>\n\n",
        )
        self.assertEqual(messages[0]["content"], "聊天 P")
