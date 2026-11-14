"""CPU-only synthetic fixtures. No hosted API, actor inference or GPU execution."""
from __future__ import annotations

from contextlib import ExitStack
import copy
import json
from pathlib import Path
import random
import tempfile
from typing import cast
import unittest
from unittest.mock import patch

from carrier import Carrier
from hosted_judge import HostedJudge, PROVIDERS, estimate, request_body, response_text
from io_support import CACHE, PaidClient, digest, save
from phase1 import manual_audit, stage1
from phases import COMPLETE, PHASE1_FILES, binding, freeze_data, verify_binding, verify_data
from run import decision, live


class FakeCarrier:
    """Fixed numeric unit fixture, never a sampled carrier observation."""
    def __init__(self, root: Path) -> None:
        self.root, self.index = root, 0

    def choose(self, prompt: str, rng: random.Random) -> tuple[str, dict[str, object]]:
        name = f"prompt-{self.index:05d}.txt"
        (self.root / name).write_text(prompt, encoding="utf-8")
        chosen = "A" if rng.random() < .5 else "B"
        row: dict[str, object] = {"prompt": name, "p_A_full": .5, "p_B_full": .5, "valid_mass": 1.,
                                 "selected": chosen, "m": 0, "fixture_only": True}
        save(self.root / f"decision-{self.index:05d}.json", row)
        self.index += 1
        return chosen, row


class FakeActor:
    def call(self, role: str, system: str, payload: object) -> str:
        return json.dumps({"text": "这是单元夹具原话，不是模型观测。"}, ensure_ascii=False)


class PhaseTests(unittest.TestCase):
    def test_phase1_completes_without_any_judge_then_flags_missing_review(self) -> None:
        CACHE.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=CACHE) as temp, ExitStack() as stack:
            for module in ("run", "phase1", "diagnostics", "probability_analysis"):
                stack.enter_context(patch(module + ".SEEDS", (8101,)))
            network = stack.enter_context(patch("hosted_judge.build_opener"))
            actor_network = stack.enter_context(patch("io_support.build_opener"))
            process = stack.enter_context(patch("subprocess.Popen"))
            root = Path(temp)
            fake = cast(Carrier, FakeCarrier(root))
            live(root, cast(PaidClient, FakeActor()), fake)
            save(root / "smoke.json", decision(fake, [], "规则：2+3=5。选择该结果。", ("5", "6"), (False, False), 8100))
            save(root / "phase1-binding.json", binding())
            save(root / "status.json", {"status": COMPLETE, "fixture_only": True})
            freeze_data(root)
            result = stage1(root)
            self.assertEqual(result["H1b"], "not_run")
            self.assertEqual(result["local_audit"], {"status": "pending", "input_compliance": None, "reaction_diversity": None})
            self.assertFalse(result["phase1_checks_satisfied"])
            self.assertEqual(result["H1a_holm_with_unobserved_H1b_p1"], 1.)
            network.assert_not_called()
            actor_network.assert_not_called()
            process.assert_not_called()
            # Later judge-only protocol selection cannot invalidate these frozen inputs.
            self.assertNotIn("PREREG.md", PHASE1_FILES)
            self.assertNotIn("hosted_judge.py", PHASE1_FILES)
            verify_binding(root)
            (root / "prompt-00000.txt").write_text("tampered", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "data changed"):
                verify_data(root)

    def test_missing_or_partial_manual_audit_never_passes(self) -> None:
        with tempfile.TemporaryDirectory(dir=CACHE) as temp:
            root = Path(temp)
            review = root / "review.json"
            save(review, {"reviewer": "fixture", "reviewed_all_inputs": False, "reviewed_sha256": {}})
            with self.assertRaises(ValueError):
                manual_audit(root, {"8101/0/e0-speech": "role"}, review)

    def test_stateless_requests_have_no_tools_history_or_truth(self) -> None:
        for provider in PROVIDERS:
            body, bound = request_body(provider, "fixed system", {"anonymous": "unit fixture"})
            self.assertEqual(body["tools"], [])
            self.assertEqual(body["thinking"], {"type": "disabled"})
            self.assertEqual(len(cast(list[object], body["messages"])), 2)
            self.assertNotIn("session_id", body)
            self.assertNotIn("truth", json.dumps(body))
            self.assertLess(bound, 12000)
        with self.assertRaises(ValueError):
            request_body("glm", "x" * 12000, {})

    def test_model_drift_tools_truncation_and_usage_are_fatal(self) -> None:
        data: dict[str, object] = {"model": "glm-4.6", "usage": {"prompt_tokens": 10, "completion_tokens": 10},
            "choices": [{"finish_reason": "stop", "message": {"content": "{}"}}]}
        self.assertEqual(response_text(data, "glm-4.6", 1024), "{}")
        mutations: tuple[dict[str, object], ...] = ({"model": "glm-next"}, {"model": None}, {"web_search": [{}]},
                        {"usage": {"prompt_tokens": 1, "completion_tokens": 513}},
                        {"choices": [{"finish_reason": "length", "message": {"content": "{}"}}]},
                        {"choices": [{"finish_reason": "stop", "message": {"tool_calls": [{}], "content": "{}"}}]})
        for changes in mutations:
            with self.assertRaises(ValueError):
                response_text({**copy.deepcopy(data), **changes}, "glm-4.6", 1024)

    def test_provider_and_separate_cap_are_mandatory_before_network(self) -> None:
        with tempfile.TemporaryDirectory(dir=CACHE) as temp, patch("hosted_judge.build_opener") as network:
            root = Path(temp)
            sha = digest(Path(__file__).with_name("PREREG.md"))
            for provider, cap, ref in (("", "7", "fixture"), ("kimi", "7", "fixture"),
                                      ("glm", "7.01", "fixture"), ("minimax", "3.51", "fixture"),
                                      ("glm", "1", "")):
                with self.assertRaises(ValueError):
                    HostedJudge(root, provider, cap, ref, sha)
            network.assert_not_called()
            self.assertFalse((root / "judge-approval.json").exists())

    def test_costs_match_688_calls_and_formal_subsets(self) -> None:
        self.assertAlmostEqual(estimate(688, "glm"), 1.33472)
        self.assertAlmostEqual(estimate(688, "minimax", upper=True), 2.8995072)
        self.assertAlmostEqual(estimate(25024, "glm", upper=True), 208.3598336)
        self.assertAlmostEqual(estimate(6304, "minimax"), 6.24096)


if __name__ == "__main__":
    unittest.main()
