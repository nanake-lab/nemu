from __future__ import annotations

import json
from typing import Any
import unittest

from check_thought import check, external_records
from think import projection_payload
from test_think import thoughts_scene


def reply(raw: str) -> dict[str, Any]:
    return dict(text=raw, done=True, finish_reason="stop", usage={"total_tokens": 12}, error=None)


class CheckTests(unittest.TestCase):
    def test_wire_input_has_only_thought_and_delivered_external_record_fields(self) -> None:
        scene = thoughts_scene()
        records = projection_payload(scene, "2026-11-05 15:25")["records"][:-1]
        binding = dict(base_records=["n1"], packs=[dict(fate="accepted", records=["e3", "n2"]),
                                                   dict(fate="rejected", records=["e4"])])
        current = "竹教我摸鱼。"
        claim = dict(span=current, kind="event", supported_by=["e3"], verdict="supported")
        result = check(current, external_records(records, binding),
                       send=lambda _: reply(json.dumps({"claims": [claim]})))
        request = result["request"]
        self.assertEqual(request["model"], "deepseek-flash")
        self.assertEqual(request["thinking"], {"type": "disabled"})
        self.assertEqual(request["response_format"], {"type": "json_object"})
        self.assertEqual([m["role"] for m in request["messages"]], ["system", "user"])
        e3 = next(r for r in records if r["id"] == "e3")
        self.assertEqual(json.loads(request["messages"][1]["content"]), dict(
            thought_text=current, external_records=[{k: e3[k] for k in ("id", "speaker", "time", "text")}]))
        self.assertFalse(result["blocked"])

    def test_mismatched_span_unknown_id_and_unmarked_guess_are_unsupported(self) -> None:
        text = "也许他来了。我记得。"
        records = [dict(id="e", speaker="竹", time="昨天", text="来了。")]
        for span, cites, verdict in [("不存在", ["e"], "supported"), ("他来了", ["missing"], "supported"),
                                     ("我记得", [], "guess"), ("他来了", [], "supported"),
                                     ("也许他来了", ["missing"], "guess")]:
            with self.subTest(span=span, cites=cites):
                raw = json.dumps(dict(claims=[dict(span=span, kind="event", supported_by=cites, verdict=verdict)]))
                result = check(text, records, send=lambda _: reply(raw))
                self.assertTrue(result["blocked"])
                self.assertEqual(result["claims"][0]["verdict"], "unsupported")
        for raw in ("{}", "not JSON", '{"claims":[]}', '{"claims":[null]}'):
            self.assertTrue(check(text, records, send=lambda _: reply(raw))["blocked"])
        guess = dict(span="也许他来了", kind="event", supported_by=[], verdict="guess")
        self.assertFalse(check(text, records, send=lambda _: reply(json.dumps(dict(claims=[guess]))))["blocked"])
        self.assertFalse(check(text, records, send=lambda _: {**reply(""), "done": False})["valid"])
