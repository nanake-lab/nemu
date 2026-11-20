from __future__ import annotations

from contextlib import redirect_stderr, redirect_stdout
from datetime import datetime
from io import StringIO
import json
from pathlib import Path
import sys
from tempfile import TemporaryDirectory
from typing import Any
import unittest
from unittest.mock import patch

from test_think import thoughts_scene
from test_think_chain import assemble
from think import main, projection_payload, thought_messages, thought_request
from think_chain import run_chains
from think_plan import THOUGHT_SPEAKER, ThoughtPlan


def point(kind: str, text: str, *cites: str) -> dict[str, Any]:
    return dict(type=kind, text=text, cites=list(cites))


def sample(*points: object) -> dict[str, Any]:
    return dict(text=json.dumps(dict(points=points), ensure_ascii=False), valid=True, usage={"total_tokens": 9})


def assembly(payload: dict[str, Any], *_: object) -> dict[str, Any]:
    assembled = assemble(payload)
    binding = assembled["binding"]
    binding["base_records"] = [payload["latest"]]
    binding["packs"][0]["records"] = [r["id"] for r in payload["records"][:-1]]
    return assembled


class PlanTests(unittest.TestCase):
    def test_roots_come_from_filtered_history_and_only_delivered_cites_are_allowed(self) -> None:
        def thought(rid: str, *sources: str) -> dict[str, Any]:
            return dict(id=rid, speaker=THOUGHT_SPEAKER, points=[point("recall", rid, *sources)])

        records = [dict(id="e", speaker="竹"), thought("a", "e"), thought("b", "a"),
                   thought("old"), thought("loop1", "e", "loop2"), thought("loop2", "loop1"),
                   dict(id="unshown", speaker="竹"), dict(id="legacy", speaker=THOUGHT_SPEAKER),
                   dict(id="now", speaker=THOUGHT_SPEAKER, points=[point("now", "夹带", "e")])]
        available = [r["id"] for r in records if r["id"] not in ("a", "unshown")]
        plan = ThoughtPlan(records, dict(base_records=[], packs=[dict(fate="accepted", records=available)]))
        result = plan.verify(sample(*(point("recall", rid, rid) for rid in
                                      ("e", "b", "old", "loop1", "loop2", "unshown", "legacy", "now")),
                                    point("recall", "混合", "e", "old")))
        self.assertEqual([p["text"] for p in result["verification"]["kept"]], ["e", "b"])
        self.assertEqual([p["reason"] for p in result["verification"]["checks"]][2:],
                         ["no_external_root", "cyclic_cite", "cyclic_cite", "undelivered_cite",
                          "no_external_root", "no_external_root", "no_external_root"])

    def test_drop_whole_points_keep_order_raw_and_base_messages(self) -> None:
        payload = projection_payload(thoughts_scene(), "2026-11-05 15:25")
        assembled = assembly(payload)
        request, plan = thought_request(assembled, payload, True, True)
        assert plan is not None
        self.assertEqual(request["messages"][:2], thought_messages(assembled["messages"]))
        self.assertNotIn("e5 =", request["messages"][2]["content"])
        self.assertEqual(request["response_format"], {"type": "json_object"})
        original = sample(point("now", "难过"), point("plan", "明天再问", "e1"),
                          point("guess", "他忘了"), point("guess", "也许没空"),
                          point("recall", "回忆"), {"type": "now", "text": "缺字段"})
        result = plan.verify(original)
        self.assertEqual(result["raw_text"], original["text"])
        self.assertEqual(result["text"], "难过。也许没空")
        self.assertEqual([c["reason"] for c in result["verification"]["checks"]],
                         [None, "unexpected_cites", "unmarked_guess", None, "empty_cites", "invalid_cites"])

    def test_chain_skips_empty_round_and_persists_points_for_later_cites(self) -> None:
        first = point("recall", "竹教过我摸鱼。", "e3")
        last = point("recall", "我记起那句教我的话。", "thought_71")
        with TemporaryDirectory() as folder, patch("think_chain.run_assembler", side_effect=assembly), \
                patch("think_chain.deepseek_sample", side_effect=[sample(first), sample(), sample(last)]) as api:
            run_chains(thoughts_scene(), datetime(2026, 11, 5, 15, 25), 10, 3, ("append",), 1,
                       "", Path(folder), plan2=True)
            self.assertEqual(api.call_count, 3)
            log, = Path(folder).glob("*.jsonl")
            rows = list(map(json.loads, log.read_text(encoding="utf-8").splitlines()))
            self.assertTrue(rows[3]["empty"])
            final = json.loads(log.with_suffix(".final.json").read_text(encoding="utf-8"))
            self.assertEqual([r["points"] for r in final["records"][len(thoughts_scene()["records"]):]],
                             [[first], [last]])

    def test_single_cli_does_not_create_append_file_for_empty_round(self) -> None:
        with TemporaryDirectory() as folder:
            output = Path(folder) / "out.json"
            args = ["think.py", "--scenario", str(Path(__file__).parent / "scenarios" / "thoughts.json"),
                    "--at", "2026-11-05 15:25", "--append", str(output), "--thinker", "deepseek", "--plan2"]
            with patch.object(sys, "argv", args), patch("think.PLAN_RUN_DIR", Path(folder)), \
                    patch.dict("os.environ", DEEPSEEK_API_KEY="test"), \
                    patch("think.run_assembler", side_effect=assembly), \
                    patch("think.deepseek_sample", return_value=sample(point("now", "旧事", "e1"))) as api, \
                    redirect_stderr(StringIO()), redirect_stdout(StringIO()):
                self.assertEqual(main(), 0)
                api.assert_called_once()
            self.assertFalse(output.exists())
            log, = Path(folder).glob("*.jsonl")
            row = json.loads(log.read_text(encoding="utf-8").splitlines()[-1])
            self.assertTrue(row["empty"])
            self.assertEqual(row["text"], "")
