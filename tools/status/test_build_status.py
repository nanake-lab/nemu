"""Offline regressions for pause precedence, reminders and board delivery."""

from __future__ import annotations

import json
import unittest
from unittest.mock import patch

import build_status as status
import serve_board


def card(cid: str = "P5-23", deps: list[str] | None = None) -> dict:
    return {
        "id": cid,
        "title": "Test card",
        "deps": deps or [],
        "difficulty": "高",
        "suggested": "Codex",
        "milestone": "N1",
    }


def pr(number: int, *, labels: list[str], draft: bool = False,
       branch: str = "task/P5-23-test", merged: bool = False) -> dict:
    return {
        "number": number,
        "title": "Test PR",
        "state": "closed" if merged else "open",
        "merged_at": "2026-10-25T00:00:00Z" if merged else None,
        "draft": draft,
        "labels": [{"name": label} for label in labels],
        "head": {"ref": branch, "sha": "synthetic-sha"},
        "html_url": f"https://github.com/example/test/pull/{number}",
        "updated_at": "2026-10-25T00:00:00Z",
    }


class PauseTests(unittest.TestCase):
    def compute(self, prs: list[dict], cards: list[dict] | None = None) -> dict:
        catalog = {
            "cards": cards if cards is not None else [card()],
            "milestones": [{"id": "N1", "name": "Test"}],
        }

        def github(path: str) -> list:
            if "/pulls?" in path:
                return prs
            if "/branches?" in path:
                return [{"name": p["head"]["ref"]} for p in prs]
            self.fail(f"Unexpected GitHub request: {path}")

        # Force every possible reminder signal, including an old queued runner.
        info = {"waiting_runner": True, "guarded": True, "verdict": None}
        with patch.object(status, "gh", side_effect=github), patch.object(
            status, "open_pr_details", return_value=info
        ):
            return status.compute("example/test", catalog)

    def test_pause_wins_over_needs_user_for_draft_and_ready(self) -> None:
        for draft in (True, False):
            with self.subTest(draft=draft):
                out = self.compute([pr(49, labels=["needs-user", "paused"], draft=draft)])
                row = status.board_rows(out)[0]
                self.assertEqual((row["status"], row["status_label"]), ("paused", "暂停"))
                self.assertEqual(row["attention"], [])
                self.assertEqual(out["attention"], [])
                self.assertEqual(out["next_up"], {})

    def test_paused_non_card_pr_has_no_reminders_and_remains_visible(self) -> None:
        for draft in (True, False):
            with self.subTest(draft=draft):
                out = self.compute([
                    pr(82, labels=["needs-user", "paused"], draft=draft, branch="docs/design")
                ], cards=[])
                self.assertEqual(out["other_attention"], [])
                self.assertEqual([p["number"] for p in out["paused_prs"]], [82])

    def test_pause_does_not_satisfy_dependencies(self) -> None:
        out = self.compute([pr(49, labels=["paused"])],
                           [card(), card("P5-24", ["P5-23"])])
        child = out["cards"][1]
        self.assertEqual(child["status"], "blocked")
        self.assertEqual(child["waiting_on"], ["P5-23"])
        self.assertEqual(out["milestones"][0]["done"], 0)

    def test_unpaused_pr_on_same_card_still_reminds(self) -> None:
        out = self.compute([
            pr(49, labels=["paused", "needs-user"]),
            pr(50, labels=["needs-user"], draft=True),
        ])
        self.assertEqual(out["cards"][0]["status"], "paused")
        self.assertEqual({a["pr"] for a in out["attention"]}, {50})
        self.assertEqual({a["kind"] for a in out["attention"]}, {"decide", "runner"})

    def test_merged_pause_label_does_not_undo_completion_or_hide_unverified(self) -> None:
        out = self.compute([pr(49, labels=["paused"], merged=True)])
        self.assertEqual(out["cards"][0]["status"], "done")
        self.assertEqual(out["paused_prs"], [])
        out = self.compute([pr(49, labels=["paused", "unverified"], merged=True)])
        self.assertEqual(out["cards"][0]["status"], "unverified")
        self.assertEqual(out["attention"][0]["kind"], "unverified")

    def test_paused_pr_skips_review_and_runner_queries(self) -> None:
        with patch.object(status, "windows_waiting") as runner, patch.object(
            status, "review_verdict"
        ) as review, patch.object(status, "touches_guarded") as guarded:
            self.assertEqual(status.open_pr_details("example/test", pr(49, labels=["paused"])), {})
        runner.assert_not_called()
        review.assert_not_called()
        guarded.assert_not_called()

    def test_local_api_delivers_card_and_non_card_pauses(self) -> None:
        out = self.compute([
            pr(49, labels=["paused"]),
            pr(82, labels=["paused"], branch="docs/design"),
        ])
        with patch.object(serve_board, "_last", None), patch.object(
            serve_board, "load_catalog", return_value={}
        ), patch.object(serve_board, "compute", return_value=out):
            code, raw = serve_board.board_json()
        body = json.loads(raw)
        self.assertEqual(code, 200)
        self.assertEqual(body["rows"][0]["status_label"], "暂停")
        self.assertEqual(body["other_attention"], [])
        self.assertEqual([p["number"] for p in body["paused_prs"]], [49, 82])


if __name__ == "__main__":
    unittest.main()
