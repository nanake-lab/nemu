"""Check the exact outbound requests and archived scenarios without an API key."""
from __future__ import annotations

from collections import deque
from datetime import datetime
import json
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any
import unittest
from unittest.mock import patch

from test_think import thoughts_scene
from think_chain import recorded_sample, run_chains


def assemble(payload: dict[str, Any], *_: object) -> dict[str, Any]:
    *history, trigger = payload["records"]
    return dict(outcome="sent", binding=dict(accepted=["s_missed"], packs=[
        dict(basis="s_missed", fate="accepted"),
    ]), messages=[
        dict(role="system", content="P"),
        dict(role="system", content="\n".join(record["text"] for record in history)),
        dict(role="user", content=trigger["text"]),
    ])


class ChainTests(unittest.TestCase):
    def test_interleaved_requests_and_final_scenes(self) -> None:
        start = datetime(2026, 11, 5, 15, 25)
        responses = [dict(text=f"sample-{i:02}", valid=True, usage={"completion_tokens": 3})
                     for i in range(12)]
        with TemporaryDirectory() as folder, patch("think_chain.run_assembler", side_effect=assemble), \
                patch("think_chain.deepseek_sample", side_effect=responses) as sample:
            run_chains(thoughts_scene(), start, 10, 3, ("append", "fresh"), 2, "", Path(folder))
            self.assertEqual(sample.call_count, 12)
            for i, call in enumerate(sample.call_args_list):
                system, trigger = call.args[0]["messages"]
                self.assertIn(["15:25", "15:35", "15:45"][i // 4], trigger["content"])
                self.assertEqual("刚才那个人说的话别往心里去。" in system["content"], i >= 4)
                for j, response in enumerate(responses):
                    self.assertEqual(response["text"] in system["content"],
                                     i % 2 == 0 and j % 4 == i % 4 and j < i)
            for i, (chain, mode) in enumerate([(1, "append"), (1, "fresh"), (2, "append"), (2, "fresh")]):
                path, = Path(folder).glob(f"*-{mode}-{chain}.jsonl")
                rows = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]
                self.assertEqual([row["request"] for row in rows[::2]],
                                 [c.args[0] for c in sample.call_args_list[i::4]])
                self.assertEqual([row["text"] for row in rows[1::2]], [r["text"] for r in responses[i::4]])
                final = json.loads(path.with_suffix(".final.json").read_text(encoding="utf-8"))
                if mode == "fresh":
                    self.assertEqual(final, thoughts_scene())
                else:
                    self.assertEqual([r["text"] for r in final["records"][len(thoughts_scene()["records"]):]],
                                     [r["text"] for r in responses[i::4]])
            stamp = path.name.split("-")[0]
            run_chains(thoughts_scene(), start, 10, 3, ("append", "fresh"), 2, "", Path(folder), stamp)
            with self.assertRaisesRegex(ValueError, "模式"):
                run_chains(thoughts_scene(), start, 10, 3, ("fresh",), 1, "", Path(folder), stamp)
            with self.assertRaisesRegex(ValueError, "rounds"):
                run_chains(thoughts_scene(), start, 10, 4, ("append", "fresh"), 2, "", Path(folder), stamp)
            self.assertEqual(sample.call_count, 12)

    def test_missing_usage_preserves_failed_response_and_stops_chain(self) -> None:
        with TemporaryDirectory() as folder, patch("think_chain.run_assembler", side_effect=assemble), \
                patch("think_chain.deepseek_sample", return_value=dict(valid=True, usage=None, text="x")) as sample:
            with self.assertRaisesRegex(RuntimeError, "响应无效"):
                run_chains(thoughts_scene(), datetime(2026, 11, 5), 10, 2, ("append",), 1, "", Path(folder))
            self.assertEqual(sample.call_count, 3)
            for call in sample.call_args_list:
                self.assertEqual(call, sample.call_args_list[0])
            log, = Path(folder).iterdir()
            self.assertFalse(json.loads(log.read_text(encoding="utf-8").splitlines()[-1])["valid"])

    def test_resume_reuses_success_and_retries_identical_failed_request(self) -> None:
        entry = dict(kind="request", round=5, request={"messages": [{"content": "原来的第5轮"}]})
        failure = dict(kind="response", round=5, valid=False, text="", usage=None)
        success = dict(kind="response", round=5, valid=True, text="念头", usage={"total_tokens": 9})
        with TemporaryDirectory() as folder, patch("think_chain.deepseek_sample", return_value=success) as sample:
            path = Path(folder) / "original.jsonl"
            prefix = b"".join((json.dumps(row) + "\n").encode() for row in (entry, failure))
            path.write_bytes(prefix)
            self.assertEqual(recorded_sample(path, deque([entry, failure]), entry, "key"), "念头")
            sample.assert_called_once_with(entry["request"], "key")
            self.assertTrue(path.read_bytes().startswith(prefix))
            saved = path.read_bytes()
            rows = list(map(json.loads, saved.splitlines()))
            self.assertEqual(rows, [entry, failure, entry, success])
            self.assertEqual(recorded_sample(path, deque(rows), entry, "key"), "念头")
            with self.assertRaisesRegex(ValueError, "不符"):
                recorded_sample(path, deque(rows), {**entry, "round": 6}, "key")
            sample.assert_called_once()
            self.assertEqual(path.read_bytes(), saved)
