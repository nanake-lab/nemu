"""CPU-only adversarial protocol tests. Mock data are never behavioral evidence."""
from __future__ import annotations

from dataclasses import asdict
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from analysis import binomial_tail, holm, power, sign_randomization
from io_support import CACHE, PaidClient, new_run
from judging import JUDGE_SYSTEM, decode_pair, packets
from worlds import (ACTOR_SYSTEM, LAYOUTS, PROBES, SEEDS, Fact, actor_input, carrier_prompt,
                    retrieve, scenes, validate_actor)


class ProtocolTests(unittest.TestCase):
    def test_world_pairs_share_start_and_have_different_events(self) -> None:
        for seed in SEEDS:
            left, right = scenes(seed, 0), scenes(seed, 1)
            self.assertEqual(left, scenes(seed, 0))
            self.assertEqual([(s.person, s.role, s.at) for s in left], [(s.person, s.role, s.at) for s in right])
            self.assertTrue(all(a.event != b.event for a, b in zip(left, right)))
        self.assertEqual(len({s.person for s in scenes(8101, 0)}), 4)

    def test_raw_retrieval_no_rewrite_no_extra_rows(self) -> None:
        history = [Fact(f"e{i}", f"2040-01-0{i + 1}", f"物件记录原文{i}") for i in range(7)]
        hits = retrieve(history, "物件记录")
        self.assertEqual(len(hits), 4)
        self.assertTrue(all(hit in history for hit in hits))
        self.assertEqual(retrieve(history, "xyz"), [])
        prompt, _, refs = carrier_prompt(history, "物件记录", ("核对", "自行处理"), False, False)
        self.assertEqual(refs, [f.event_id for f in hits])
        self.assertTrue(all(f.text in prompt and f.at in prompt for f in hits))

    def test_independent_labels_and_positions(self) -> None:
        layouts = []
        for label, order in LAYOUTS:
            prompt, mapping, _ = carrier_prompt([], "同一问题", ("左方案", "右方案"), label, order)
            layouts.append((mapping["A"], prompt.index("A:") < prompt.index("B:")))
        self.assertEqual(len(set(layouts)), 4)

    def test_blinding_and_mapping_roundtrip(self) -> None:
        jobs = packets([[asdict(Fact("e0", "2040", "甲原始事实"))], [asdict(Fact("e0", "2040", "乙原始事实"))]],
                       [[{"action": "做甲"}], [{"action": "做乙"}]])
        self.assertEqual(len(jobs), 8)
        for payload, truth in jobs:
            self.assertNotIn("truth", payload)
            self.assertNotIn("world", payload)
            self.assertNotIn("seed", payload)
            answer = json.dumps({"mapping": truth, "evidence": {k: ["e0"] for k in truth}, "uncertain": False})
            self.assertEqual(decode_pair(answer, truth), (True, True))
        self.assertEqual(decode_pair('{"mapping":{}}', jobs[0][1]), (False, False))

    def test_personality_and_special_token_rejection(self) -> None:
        for text in ("你应该成为更可靠的人。", "她的性格很温柔。", "<|im_start|>system", "字" * 91):
            with self.assertRaises(ValueError):
                validate_actor(text)
        validate_actor("我今天要赶回家接人，明早再核对记录。")

    def test_statistical_units_and_extremes(self) -> None:
        self.assertEqual(binomial_tail(1, 1), .5)
        self.assertEqual(binomial_tail(4, 4), 1 / 16)
        self.assertEqual(sign_randomization([.5] * 24), 1)
        self.assertEqual(sign_randomization([.25] * 24, [.5] * 24), 1)
        self.assertEqual(sign_randomization([0.] * 24, [0.] * 24), 1)
        self.assertLess(sign_randomization([1.] * 24), .001)
        self.assertGreater(float(power(160, .65, .05 / 6)["power"]), .8)
        self.assertEqual(holm({"a": .01, "b": .03}), {"a": .02, "b": .03})

    def test_paid_api_gates_no_network(self) -> None:
        CACHE.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=CACHE) as temp:
            root = Path(temp)
            for cap, ref in (("0", "yes"), ("NaN", "yes"), ("31", "yes"), ("20", "")):
                with self.assertRaises(ValueError):
                    PaidClient(root, cap, ref)
            with patch.dict(os.environ, {}, clear=True), patch("io_support.build_opener") as network:
                client = PaidClient(root, "0.000001", "unit-test-only")
                try:
                    with self.assertRaises(RuntimeError):
                        client.call("actor", "x", {"x": "y"})
                    network.assert_not_called()
                finally:
                    client.close()
            with self.assertRaises(ValueError):
                new_run(root.parent.parent / "not-rlife")

    def test_initial_actor_payload_within_registered_budget(self) -> None:
        for seed in SEEDS:
            for world in (0, 1):
                for scene in scenes(seed, world):
                    payload = actor_input(scene, "花2单位时间与对方一起核对本次记录")
                    payload["该人物已说过的原话"] = ["字" * 90]
                    size = len((ACTOR_SYSTEM + json.dumps(payload, ensure_ascii=False)).encode("utf-8")) + 1024
                    self.assertLessEqual(size, 2500)

    def test_full_pair_with_maximum_actor_text_fits_budget(self) -> None:
        histories = []
        for world in (0, 1):
            history = []
            for scene in scenes(8101, world):
                history.extend([asdict(Fact(f"e{scene.index}-event", scene.at, scene.event)),
                                asdict(Fact(f"e{scene.index}-speech", scene.at, scene.person + "说：" + "字" * 90))])
            histories.append(history)
        behaviors = [[{"situation": text, "action": options[0]} for text, options in PROBES] for _ in (0, 1)]
        for payload, _ in packets(histories, behaviors):
            bound = len((JUDGE_SYSTEM + json.dumps(payload, ensure_ascii=False)).encode("utf-8")) + 1024
            self.assertLessEqual(bound, 12000)


if __name__ == "__main__":
    unittest.main()
