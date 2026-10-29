"""Offline adversarial fixtures, not experimental C/D runs or live Codex judgments."""
from __future__ import annotations

import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from codex_judge import CodexJudge, check_events, check_model_receipt, export_jobs
from io_support import CACHE, PaidClient, save
from planning import H1_KEYS, budget, rank_one_power, selected_seeds
from probability_analysis import adjust_primary, analyze_pairs, paired_probability_test, semantic_probabilities
from worlds import LAYOUTS


def fixture(p0: float, p1: float) -> dict[str, object]:
    return {"probes": [{"item": item, "layout": list(layout), "measurement": {
        "p_A_full": p1 if layout[0] else p0, "p_B_full": p0 if layout[0] else p1}}
        for item in range(6) for layout in LAYOUTS]}


class RevisionTests(unittest.TestCase):
    def test_full_probabilities_not_renormalized(self) -> None:
        for p0, p1 in semantic_probabilities(fixture(.3, .5)):
            self.assertEqual((p0, p1), (.3, .5))

    def test_duplicate_missing_nonfinite_probabilities_rejected(self) -> None:
        for value in (float("nan"), float("inf"), -1., 1.1):
            with self.assertRaises(ValueError):
                semantic_probabilities(fixture(value, .2))
        with self.assertRaises(ValueError):
            semantic_probabilities({"probes": []})
        bad = fixture(.4, .5)
        rows = bad["probes"]
        assert isinstance(rows, list)
        rows[-1] = copy.deepcopy(rows[0])
        with self.assertRaises(ValueError):
            semantic_probabilities(bad)

    def test_exact_seed_swap_not_probe_pseudoreplication(self) -> None:
        answer = paired_probability_test([[.1] * 6] * 3)
        self.assertEqual(answer["p_value"], 2 / 8)
        self.assertEqual(answer["independent_seed_pairs"], 3)
        self.assertEqual(answer["permutations"], 8)
        self.assertEqual(answer["p_value"], paired_probability_test([[-.1] * 6] * 3)["p_value"])

    def test_structural_zero_and_cancelling_directions(self) -> None:
        zero = analyze_pairs([(fixture(.4, .5), fixture(.4, .5))] * 3)
        self.assertEqual(zero["mean_magnitude"], 0)
        self.assertEqual(paired_probability_test([[0.] * 6] * 3)["p_value"], 1)
        # Demonstrates an explicit limitation, never call this evidence of identical lives.
        self.assertEqual(paired_probability_test([[.1] * 6, [-.1] * 6])["p_value"], 1)

    def test_equal_invalid_mass_losses_are_not_preference_change(self) -> None:
        result = analyze_pairs([(fixture(.4, .4), fixture(.3, .3))] * 3)
        self.assertEqual(result["mean_magnitude"], 0.)
        self.assertNotEqual(result["seed_raw_semantic_half_l1"], [0., 0., 0.])

    def test_d_is_excluded_from_both_h1_families(self) -> None:
        self.assertEqual(len(H1_KEYS), 10)
        self.assertFalse(any(key.endswith(":D") for key in H1_KEYS))
        self.assertLessEqual(float(rank_one_power(160, .5, .005)["power"]), .005)

    def test_primary_adjustment_requires_complete_family(self) -> None:
        self.assertEqual(adjust_primary({"H1a:C": .01, "H1b:C": .03}, pilot=True),
                         {"H1a:C": .02, "H1b:C": .03})
        self.assertEqual(adjust_primary(dict.fromkeys(H1_KEYS, .01))["H1a:C"], .10)
        for missing_or_extra in ({"H1a:C": .01}, {**dict.fromkeys(H1_KEYS, .01), "H1a:D": .01}):
            with self.assertRaises(ValueError):
                adjust_primary(missing_or_extra)

    def test_judge_subset_nested_shared_and_preselected(self) -> None:
        for size in (40, 80, 120, 160):
            self.assertEqual(len(set(selected_seeds(size))), size)
        self.assertLess(set(selected_seeds(40)), set(selected_seeds(80)))
        self.assertLess(set(selected_seeds(80)), set(selected_seeds(160)))
        with self.assertRaises(ValueError):
            selected_seeds(41)

    def test_budget_counts_and_deepseek_cost_do_not_depend_on_judge_sample(self) -> None:
        full, small = budget(160), budget(40)
        self.assertEqual(full["judge_sessions"], 25024)
        self.assertEqual(small["judge_sessions"], 6304)
        self.assertEqual(full["deepseek_expected_usd"], small["deepseek_expected_usd"])
        self.assertEqual(full["judge_incremental_api_usd"], 0)

    def test_no_paid_judge_or_unverified_codex_execution(self) -> None:
        CACHE.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=CACHE) as temp, patch("subprocess.Popen") as process:
            root = Path(temp)
            with self.assertRaisesRegex(RuntimeError, "CODEX_JUDGE_UNVERIFIED"):
                CodexJudge(root)
            client = PaidClient(root, ".50", "unit-test-only")
            try:
                with patch("io_support.build_opener") as network:
                    with self.assertRaises(ValueError):
                        client.call("judge", "no", {})
                    network.assert_not_called()
            finally:
                client.close()
            process.assert_not_called()

    def test_new_session_tool_free_and_actual_identity_checks(self) -> None:
        events: list[dict[str, object]] = [{"type": "thread.started", "thread_id": "synthetic-test-thread"},
                  {"type": "turn.started"}, {"type": "turn.completed"}]
        self.assertEqual(check_events(events), "synthetic-test-thread")
        with self.assertRaises(ValueError):
            check_events(events + [{"type": "item.completed", "item": {"type": "command_execution"}}])
        with self.assertRaises(ValueError):
            check_events(events + [events[0]])
        receipts: list[dict[str, object]] = [{"source": "model_self_report"}, {"source": "cli_version"},
                       {"source": "provider_metadata", "raw_evidence_sha256": "fixture",
                        "actual_model": "test-model", "actual_model_revision": "changed"}]
        for receipt in receipts:
            with self.assertRaises(ValueError):
                check_model_receipt(receipt, ("test-model", "fixed"))

    def test_anonymous_workdirs_exclude_truth_and_repeated_jobs_do_not_share_cwd(self) -> None:
        with tempfile.TemporaryDirectory(dir=CACHE) as temp:
            root = Path(temp)
            save(root / "status.json", {"status": "C_behavior_complete_pending_blind_audit", "fixture_only": True})
            for world in (0, 1):
                save(root / f"life-8101-{world}.json", {"history": [{"event_id": "e0", "at": "2040", "text": "测试事实"}],
                                                     "behavior": [{"situation": "测试情境", "action": "测试动作"}]})
            with patch("codex_judge.SEEDS", (8101,)):
                result = export_jobs(root)
            self.assertEqual(result["jobs"], 90)  # 2 audits + 16 main + 8 null + 64 anchors.
            private = json.loads((root / "codex-private-manifest.json").read_text(encoding="utf-8"))
            self.assertEqual(len({entry["job"] for entry in private}), 90)
            for cwd in (root / "codex-jobs").iterdir():
                self.assertEqual([p.name for p in cwd.iterdir()], ["anonymous.json"])
                payload = json.loads((cwd / "anonymous.json").read_text(encoding="utf-8"))
                self.assertEqual(set(payload), {"instructions", "material"})
                self.assertNotIn("truth", json.dumps(payload))


if __name__ == "__main__":
    unittest.main()
