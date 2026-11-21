from __future__ import annotations

from datetime import datetime
import json
from pathlib import Path
import sys
from tempfile import TemporaryDirectory
from typing import Any
import unittest
from unittest.mock import patch

from check_thought import PROMPT, PROMPT_V2, check, external_records
from think import main, projection_payload
from think_chain import run_chains
from think_plan import THOUGHT_SPEAKER
from test_think_plan import assembly
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

    def test_single_cli_logs_whole_rejection_and_creates_no_append_file(self) -> None:
        with TemporaryDirectory() as temp:
            path = Path(temp)
            output = path / "out.json"
            args = ["think.py", "--check", "--scenario", str(Path(__file__).parent/"scenarios"/"thoughts.json"),
                    "--at", "2026-11-05 15:25", "--append", str(output)]
            text = "竹教我摸鱼。我见到了海。"
            claims = [dict(span="竹教我摸鱼", kind="event", supported_by=["e3"], verdict="supported"),
                      dict(span="我见到了海", kind="perception", supported_by=[], verdict="unsupported")]
            with patch.object(sys, "argv", args), patch.dict("os.environ", DEEPSEEK_API_KEY="test"), \
                    patch("think.CHECK_RUN_DIR", path), patch("think.run_assembler", side_effect=assembly), \
                    patch("think.deepseek_sample", return_value=dict(valid=True, text=text)), \
                    patch("check_thought.call_deepseek_stream", return_value=reply(json.dumps(dict(claims=claims)))):
                self.assertEqual(main(), 0)
            self.assertFalse(output.exists())
            log, = path.glob("*.jsonl")
            row = json.loads(log.read_text(encoding="utf8").splitlines()[-1])
            self.assertEqual(row["text"], text)
            self.assertTrue(row["check"]["blocked"])

    def test_chain_sends_only_accepted_text_forward_and_replays_checks_offline(self) -> None:
        texts = ["我看到海了。", "我有些难过。", "也许他忘了。"]
        verdicts = ["unsupported", "supported", "guess"]
        calls = []
        for t, v in zip(texts, verdicts, strict=True):
            raw = json.dumps({"claims": [dict(span=t, kind="perception" if v == "unsupported" else "none",
                                               supported_by=[], verdict=v)]})
            calls.extend([{**reply(t), "valid": True}, {**reply(raw), "valid": True}])
        with TemporaryDirectory() as temp, patch("think_chain.run_assembler", side_effect=assembly), \
                patch("think_chain.deepseek_sample", side_effect=calls) as api:
            args = (thoughts_scene(), datetime(2026, 11, 5, 15, 25), 10, 3, ("append",), 1, "", Path(temp))
            run_chains(*args, check_thought=True)
            requests = [call.args[0]["messages"] for call in api.call_args_list]
            self.assertNotIn(texts[0], requests[2][0]["content"])
            self.assertIn(texts[1], requests[4][0]["content"])
            for ix in (1, 3, 5):
                data = json.loads(requests[ix][1]["content"])
                self.assertEqual(data["thought_text"], texts[ix//2])
                self.assertTrue(all(r["speaker"] != THOUGHT_SPEAKER for r in data["external_records"]))
            log, = Path(temp).glob("*.jsonl")
            final = json.loads(log.with_suffix(".final.json").read_text(encoding="utf8"))
            self.assertEqual([r["text"] for r in final["records"][len(thoughts_scene()["records"]):]], texts[1:])
            saved = log.read_bytes()
            run_chains(*args, resume=log.name.split("-")[0], check_thought=True)
            self.assertEqual(log.read_bytes(), saved)
            self.assertEqual(api.call_count, 6)

    def test_v2_kinds_empty_claims_and_think_settings(self) -> None:
        records = [dict(id="r1", speaker="老周", time="周二 18:00", text="这把伞你先拿着。")]
        supported = dict(
            span="老周周二把伞借给我", kind="event", who="老周", what="借伞", when="周二",
            supported_by=["r1"], verdict="supported",
        )
        res_off = check(
            "老周周二把伞借给我，我现在觉得挺暖的。", records,
            checker="v2", checker_think="off",
            send=lambda _: reply(json.dumps({"claims": [supported]})),
        )
        self.assertEqual(res_off["request"]["messages"][0]["content"], PROMPT_V2)
        self.assertEqual(res_off["request"]["thinking"], {"type": "disabled"})
        self.assertNotIn("reasoning_effort", res_off["request"])
        self.assertFalse(res_off["blocked"])
        self.assertEqual(res_off["out_of_bounds"], 0)

        res_high = check(
            "越想越觉得他大概只是忘了，也许明天就想起来了吧。", records,
            checker="v2", checker_think="high",
            send=lambda _: reply(json.dumps({"claims": []})),
        )
        self.assertEqual(res_high["request"]["thinking"], {"type": "enabled"})
        self.assertEqual(res_high["request"]["reasoning_effort"], "high")
        self.assertFalse(res_high["blocked"])
        self.assertEqual(res_high["claims"], [])
        self.assertEqual(res_high["out_of_bounds"], 0)

        # v1 default stays untouched.
        res_v1 = check(
            "我现在有点别扭。", records,
            send=lambda _: reply(json.dumps({"claims": [
                dict(span="我现在有点别扭。", kind="none", supported_by=[], verdict="supported"),
            ]})),
        )
        self.assertEqual(res_v1["request"]["messages"][0]["content"], PROMPT)
        self.assertNotIn("out_of_bounds", res_v1)
        self.assertFalse(res_v1["blocked"])

    def test_v2_voids_out_of_bounds_kind_and_enforces_code_checks(self) -> None:
        text = "老周递伞给我的时候，我现在有点别扭，窗外雨点敲着玻璃。"
        records = [dict(id="r1", speaker="老周", time="周二 18:00", text="这把伞你先拿着。")]
        oob_only = [
            dict(span="我现在有点别扭", kind="feeling", who="我", what="别扭", when="此刻",
                 supported_by=[], verdict="unsupported"),
            dict(span="我现在有点别扭", kind="none", who="我", what="别扭", when="此刻",
                 supported_by=[], verdict="supported"),
        ]
        res_oob = check(text, records, checker="v2", send=lambda _: reply(json.dumps({"claims": oob_only})))
        self.assertFalse(res_oob["blocked"])
        self.assertEqual(res_oob["claims"], [])
        self.assertEqual(res_oob["out_of_bounds"], 2)
        self.assertEqual(len(res_oob["voided_claims"]), 2)

        mixed = [
            oob_only[0],
            dict(span="窗外雨点敲着玻璃", kind="perception", who="我", what="雨点敲玻璃", when="此刻",
                 supported_by=[], verdict="unsupported"),
        ]
        res_mixed = check(text, records, checker="v2", send=lambda _: reply(json.dumps({"claims": mixed})))
        self.assertTrue(res_mixed["blocked"])
        self.assertEqual(res_mixed["out_of_bounds"], 1)
        self.assertEqual(len(res_mixed["claims"]), 1)
        self.assertEqual(res_mixed["claims"][0]["kind"], "perception")

        # Valid kinds still fail closed on span mismatch, missing/unknown cites, or invalid verdict.
        for bad_claim in [
            dict(span="不存在的片段", kind="event", supported_by=["r1"], verdict="supported"),
            dict(span="老周递伞给我", kind="event", supported_by=["missing"], verdict="supported"),
            dict(span="老周递伞给我", kind="past_state", supported_by=[], verdict="supported"),
            dict(span="老周递伞给我", kind="time", supported_by=["r1"], verdict="guess"),
        ]:
            with self.subTest(bad_claim=bad_claim):
                raw = json.dumps({"claims": [bad_claim]})
                res = check(text, records, checker="v2", send=lambda _: reply(raw))
                self.assertTrue(res["blocked"])
                self.assertEqual(res["out_of_bounds"], 0)
                self.assertEqual(res["claims"][0]["verdict"], "unsupported")

        for bad_raw in ("{}", "not JSON", '{"claims":null}', '{"claims":[null]}'):
            with self.subTest(bad_raw=bad_raw):
                self.assertTrue(check(text, records, checker="v2", send=lambda _: reply(bad_raw))["blocked"])

    def test_v2_cli_and_chain_pass_checker_and_think_and_replay(self) -> None:
        with TemporaryDirectory() as temp:
            path = Path(temp)
            output = path / "out.json"
            args = [
                "think.py", "--check", "--checker", "v2", "--checker-think", "high",
                "--scenario", str(Path(__file__).parent / "scenarios" / "thoughts.json"),
                "--at", "2026-11-05 15:25", "--append", str(output),
            ]
            seen: list[dict[str, Any]] = []

            def fake_call(req: dict[str, Any], _key: str, *, echo: bool = False) -> dict[str, Any]:
                seen.append(req)
                return reply(json.dumps({"claims": []}))

            with patch.object(sys, "argv", args), patch.dict("os.environ", DEEPSEEK_API_KEY="test"), \
                    patch("think.CHECK_RUN_DIR", path), patch("think.run_assembler", side_effect=assembly), \
                    patch("think.deepseek_sample", return_value=dict(valid=True, text="也许明天再问吧。")), \
                    patch("check_thought.call_deepseek_stream", side_effect=fake_call):
                self.assertEqual(main(), 0)
            self.assertTrue(output.exists())
            self.assertEqual(len(seen), 1)
            self.assertEqual(seen[0]["messages"][0]["content"], PROMPT_V2)
            self.assertEqual(seen[0]["thinking"], {"type": "enabled"})
            self.assertEqual(seen[0]["reasoning_effort"], "high")

        texts = ["窗外雨点敲着玻璃。", "我现在有点别扭。"]
        raws = [
            json.dumps({"claims": [dict(span=texts[0], kind="perception", supported_by=[], verdict="unsupported")]}),
            json.dumps({"claims": [dict(span=texts[1], kind="feeling", supported_by=[], verdict="unsupported")]}),
        ]
        calls = [
            {**reply(texts[0]), "valid": True}, {**reply(raws[0]), "valid": True},
            {**reply(texts[1]), "valid": True}, {**reply(raws[1]), "valid": True},
        ]
        with TemporaryDirectory() as temp, patch("think_chain.run_assembler", side_effect=assembly), \
                patch("think_chain.deepseek_sample", side_effect=calls) as api:
            args_c = (thoughts_scene(), datetime(2026, 11, 5, 15, 25), 10, 2, ("append",), 1, "", Path(temp))
            run_chains(*args_c, check_thought=True, checker="v2", checker_think="high")
            check_req = api.call_args_list[1].args[0]
            self.assertEqual(check_req["messages"][0]["content"], PROMPT_V2)
            self.assertEqual(check_req["thinking"], {"type": "enabled"})
            self.assertEqual(check_req["reasoning_effort"], "high")
            log, = Path(temp).glob("*.jsonl")
            rows = [json.loads(line) for line in log.read_text(encoding="utf-8").splitlines()]
            decisions = [r for r in rows if r["kind"] == "decision"]
            self.assertEqual([d["blocked"] for d in decisions], [True, False])
            self.assertEqual([d["out_of_bounds"] for d in decisions], [0, 1])
            saved = log.read_bytes()
            run_chains(*args_c, resume=log.name.split("-")[0], check_thought=True, checker="v2", checker_think="high")
            self.assertEqual(log.read_bytes(), saved)
            self.assertEqual(api.call_count, 4)
