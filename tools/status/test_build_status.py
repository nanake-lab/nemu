"""Offline regressions for pause precedence, reminders and board delivery."""

from __future__ import annotations

import json
import unittest
from unittest.mock import patch

import build_status as status
import serve_board
from status_types import Board, Card, Catalog, RawPr


def card(cid: str = "P5-23", deps: list[str] | None = None) -> Card:
    return {
        "id": cid,
        "title": "Test card",
        "deps": deps or [],
        "difficulty": "高",
        "suggested": "Codex",
        "milestone": "N1",
    }


def pr(number: int, *, labels: list[str], draft: bool = False,
       branch: str = "task/P5-23-test", merged: bool = False) -> RawPr:
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
    def compute(self, prs: list[RawPr], cards: list[Card] | None = None,
                deliveries: dict[str, list[str]] | None = None) -> Board:
        catalog: Catalog = {
            "cards": cards if cards is not None else [card()],
            "milestones": [{"id": "N1", "name": "Test"}],
        }

        def github(path: str) -> list[RawPr] | list[dict[str, str]]:
            if "/pulls?" in path:
                return prs
            if "/branches?" in path:
                return [{"name": p["head"]["ref"]} for p in prs]
            self.fail(f"Unexpected GitHub request: {path}")

        # Force every possible reminder signal, including an old queued runner.
        info = {"waiting_runner": True, "guarded": True, "verdict": None}
        with patch.object(status, "gh", side_effect=github), patch.object(
            status, "open_pr_details", return_value=info
        ), patch.object(status, "load_deliveries", return_value=deliveries or {}):
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

    def test_merged_delivery_controls_completion_and_dependencies(self) -> None:
        # BOARD-PARTIAL-01: a merged slice must not complete its card or unlock dependants.
        for deliveries, expected in (({}, "done"), ({"P5-23": ["P5-06"]}, "partial")):
            with self.subTest(expected=expected):
                out = self.compute([pr(49, labels=[], merged=True)],
                                   [card(), card("P5-24", ["P5-23"])], deliveries)
                row = status.board_rows(out)[0]
                self.assertEqual(row["status"], expected)
                self.assertEqual(row["status_label"], status.LABELS[expected])
                self.assertEqual(row["waiting_on"], "P5-06" if deliveries else "")
                self.assertEqual(out["cards"][1]["status"], "blocked" if deliveries else "ready")
                self.assertEqual(out["milestones"][0]["done"], 0 if deliveries else 1)

    def test_partial_remainder_waiting_on_dependant_does_not_block_it(self) -> None:
        # The remainder of P5-23 waits on P5-24 (directly) or on P5-25, which
        # needs P5-24: P5-24 can only use the delivered part, so it may start.
        cards = [card(), card("P5-24", ["P5-23"]), card("P5-25", ["P5-24"])]
        for remainder in (["P5-24"], ["P5-25", "请求"]):
            with self.subTest(remainder=remainder):
                out = self.compute([pr(49, labels=[], merged=True)], cards,
                                   {"P5-23": remainder})
                by_id = {c["id"]: c for c in out["cards"]}
                self.assertEqual(by_id["P5-23"]["status"], "partial")
                self.assertEqual(by_id["P5-24"]["status"], "ready")
                self.assertEqual(by_id["P5-24"]["waiting_on"], [])
                self.assertEqual(by_id["P5-25"]["status"], "blocked")
                self.assertEqual(by_id["P5-25"]["waiting_on"], ["P5-24"])
                self.assertEqual(out["milestones"][0]["done"], 0)

    def test_closed_card_override_drops_old_remainder(self) -> None:
        closed = card()
        closed["status_override"] = {"status": "done", "note": "用户结案"}
        out = self.compute([pr(49, labels=[], merged=True)],
                           [closed, card("P5-24", ["P5-23"])],
                           {"P5-23": ["见 reports/P5-23/BLOCKED.md"]})
        by_id = {c["id"]: c for c in out["cards"]}
        self.assertEqual((by_id["P5-23"]["status"], by_id["P5-23"]["waiting_on"]), ("done", []))
        self.assertEqual(by_id["P5-24"]["status"], "ready")

    def test_partial_does_not_hide_open_work_or_unverified_merge(self) -> None:
        for prs, expected in (
            ([pr(49, labels=["unverified"], merged=True)], "unverified"),
            ([pr(49, labels=[], merged=True), pr(50, labels=[], draft=True)], "in_progress"),
        ):
            with self.subTest(expected=expected):
                out = self.compute(prs, deliveries={"P5-23": ["P5-06"]})
                self.assertEqual(out["cards"][0]["status"], expected)
                self.assertEqual(out["cards"][0]["waiting_on"], ["P5-06"])

    def test_paused_non_card_pr_has_no_reminders_and_remains_visible(self) -> None:
        for draft in (True, False):
            with self.subTest(draft=draft):
                out = self.compute([
                    pr(82, labels=["needs-user", "paused"], draft=draft, branch="docs/design")
                ], cards=[])
                self.assertEqual(out["other_attention"], [])
                self.assertEqual([p["number"] for p in out["paused_prs"]], [82])

    def test_card_edit_pr_neither_claims_nor_finishes_card(self) -> None:
        for merged in (False, True):
            with self.subTest(merged=merged):
                out = self.compute(
                    [pr(139, labels=["needs-user", "card-edit"], merged=merged,
                        branch="task/P5-23-card-sync")],
                    cards=[card(), card("P5-24", ["P5-23"])],
                )
                cards = {c["id"]: c for c in out["cards"]}
                self.assertEqual(cards["P5-23"]["status"], "ready")
                self.assertEqual(cards["P5-23"]["prs"], [])
                self.assertEqual(cards["P5-24"]["status"], "blocked")

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
