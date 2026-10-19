#!/usr/bin/env python3
"""Derive every task card's status from GitHub and write status.json.

Nobody edits card status by hand. The catalog (tasks/cards.json) holds static
facts; this script infers status from pull requests and branches, so the board
cannot go stale and parallel PRs never conflict on a status file.

Usage (repository root, GitHub CLI authenticated):
    python tools/status/build_status.py [--out status.json] [--board board.json] [--repo owner/name]

Rules, first match wins:
    status_override in the catalog       -> that status
    merged PR with label unverified      -> unverified (merged, cross-review unfinished;
                                            dependants stay blocked)
    open PR with label needs-user        -> needs_you
    open draft PR                        -> in_progress (claimed)
    open ready PR                        -> review
    merged PR                            -> done
    branch task/<id>-* without any PR    -> in_progress (claimed)
    all dependencies done, card is yours -> needs_you
    all dependencies done                -> ready
    otherwise                            -> blocked

Review (AGENTS.md section 6): medium and high cards need a review comment whose
first line is "审查结论：通过" or "审查结论：有阻塞问题". Claude reviews unless
Claude is the suggested executor, in which case Codex does; the board asks the
user to forward a PR only when Codex is the reviewer.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import re
import subprocess
from pathlib import Path

CARD = re.compile(r"\((P\d-\d{2})\)")
BRANCH = re.compile(r"^task/(P\d-\d{2})\b")


def gh(path: str) -> list | dict:
    raw = subprocess.run(
        ["gh", "api", "--paginate", "--slurp", path],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    pages = json.loads(raw)
    if pages and isinstance(pages[0], list):
        return [item for page in pages for item in page]
    return pages[0] if pages else {}


def cards_of_pr(pr: dict) -> set[str]:
    found = set()
    m = BRANCH.match(pr["head"]["ref"])
    if m:
        found.add(m.group(1))
    found.update(CARD.findall(pr["title"]))
    return found


def windows_waiting(repo: str, sha: str) -> bool:
    runs = gh(f"repos/{repo}/actions/runs?head_sha={sha}&per_page=5")
    runs = runs.get("workflow_runs", []) if isinstance(runs, dict) else []
    if not runs:
        return False
    jobs = gh(f"repos/{repo}/actions/runs/{runs[0]['id']}/jobs")
    jobs = jobs.get("jobs", []) if isinstance(jobs, dict) else []
    return any(
        "Windows" in j["name"] and j["status"] in ("queued", "waiting") for j in jobs
    )


VERDICT = "审查结论："


def review_verdict(repo: str, number: int) -> str | None:
    """'pass', 'block' or None, from the latest review comment on the PR."""
    comments = gh(f"repos/{repo}/issues/{number}/comments?per_page=100")
    verdict = None
    for cm in comments if isinstance(comments, list) else []:
        body = (cm.get("body") or "").lstrip()
        if body.startswith(VERDICT):
            verdict = "pass" if body.startswith(VERDICT + "通过") else "block"
    return verdict


def reviewer(card: dict) -> str:
    return "Codex" if card["suggested"] == "Claude" else "Claude"


def label_of(card: dict) -> str:
    if card["status"] != "review":
        return LABELS[card["status"]]
    if card["difficulty"] == "低":
        return "等合并"
    verdicts = {
        p.get("verdict") for p in card["prs"] if p["state"] == "open" and not p["draft"]
    }
    if "pass" in verdicts:
        return "审查通过·待合并"
    if "block" in verdicts:
        return "审查有阻塞·待修改"
    return f"等 {reviewer(card)} 审查"


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default="nanake-lab/nemu")
    ap.add_argument("--catalog", default="tasks/cards.json")
    ap.add_argument("--out", default="status.json")
    ap.add_argument(
        "--board", help="also write the web board's row file (one row per card)"
    )
    args = ap.parse_args()

    catalog = json.loads(Path(args.catalog).read_text(encoding="utf-8"))
    cards = {c["id"]: dict(c) for c in catalog["cards"]}
    prs = gh(f"repos/{args.repo}/pulls?state=all&per_page=100")
    branches = [b["name"] for b in gh(f"repos/{args.repo}/branches?per_page=100")]

    for c in cards.values():
        c["prs"] = []
    for pr in prs:
        for cid in cards_of_pr(pr) & cards.keys():
            state = "merged" if pr.get("merged_at") else pr["state"]
            if state == "closed":
                continue
            entry = {
                "number": pr["number"],
                "url": pr["html_url"],
                "title": pr["title"],
                "state": state,
                "draft": bool(pr.get("draft")),
                "labels": [lbl["name"] for lbl in pr.get("labels", [])],
                "updated_at": pr["updated_at"],
            }
            if state == "open":
                entry["waiting_runner"] = windows_waiting(args.repo, pr["head"]["sha"])
                if not entry["draft"]:
                    entry["verdict"] = review_verdict(args.repo, pr["number"])
            cards[cid]["prs"].append(entry)
    claimed = {m.group(1) for b in branches if (m := BRANCH.match(b))}

    def direct(c: dict) -> str | None:
        if c.get("status_override"):
            return c["status_override"]["status"]
        open_prs = [p for p in c["prs"] if p["state"] == "open"]
        if any("needs-user" in p["labels"] for p in open_prs):
            return "needs_you"
        if any(p["draft"] for p in open_prs):
            return "in_progress"
        if open_prs:
            return "review"
        if any(
            p["state"] == "merged" and "unverified" in p["labels"] for p in c["prs"]
        ):
            return "unverified"
        if any(p["state"] == "merged" for p in c["prs"]):
            return "done"
        if c["id"] in claimed:
            return "in_progress"
        return None

    status: dict[str, str] = {}
    for cid in sorted(cards):
        s = direct(cards[cid])
        if s:
            status[cid] = s
    changed = True
    while changed:
        changed = False
        for cid, c in sorted(cards.items()):
            if cid in status:
                continue
            if all(status.get(d) == "done" for d in c["deps"]):
                status[cid] = "needs_you" if c["suggested"] == "你" else "ready"
                changed = True
    for cid, card in cards.items():
        card["status"] = status.setdefault(cid, "blocked")
        card["waiting_on"] = [d for d in card["deps"] if status.get(d) != "done"]
        card["status_label"] = label_of(card)

    attention = []
    for c in sorted(cards.values(), key=lambda c: c["id"]):
        open_prs = [p for p in c["prs"] if p["state"] == "open"]
        for p in open_prs:
            if "needs-user" in p["labels"]:
                attention.append(
                    {
                        "card": c["id"],
                        "kind": "approve",
                        "pr": p["number"],
                        "url": p["url"],
                        "text": f"PR #{p['number']} 需要你批准",
                    }
                )
            if p.get("waiting_runner"):
                attention.append(
                    {
                        "card": c["id"],
                        "kind": "runner",
                        "pr": p["number"],
                        "url": p["url"],
                        "text": f"PR #{p['number']} 的 Windows 检查在排队，需要开 runner",
                    }
                )
            if (
                c["difficulty"] in ("中", "高")
                and reviewer(c) == "Codex"
                and not p["draft"]
                and "needs-user" not in p["labels"]
                and p.get("verdict") is None
            ):
                attention.append(
                    {
                        "card": c["id"],
                        "kind": "cross_review",
                        "pr": p["number"],
                        "url": p["url"],
                        "text": f"PR #{p['number']}（{c['id']}，{c['difficulty']}难度）请转给 Codex 审查",
                    }
                )
        for p in c["prs"]:
            if p["state"] == "merged" and "unverified" in p["labels"]:
                attention.append(
                    {
                        "card": c["id"],
                        "kind": "unverified",
                        "pr": p["number"],
                        "url": p["url"],
                        "text": f"{c['id']} 已合并但交叉审查没做完（PR #{p['number']}），需要补完核验",
                    }
                )
        if c["status"] == "needs_you" and c["suggested"] == "你":
            attention.append(
                {
                    "card": c["id"],
                    "kind": "your_card",
                    "text": f"{c['id']} {c['title']} 可以开始了",
                }
            )

    milestones = []
    for m in catalog["milestones"]:
        ids = [cid for cid, c in cards.items() if c["milestone"] == m["id"]]
        done = sum(1 for cid in ids if cards[cid]["status"] == "done")
        milestones.append(
            {
                **m,
                "total": len(ids),
                "done": done,
                "remaining": sorted(
                    cid for cid in ids if cards[cid]["status"] != "done"
                ),
            }
        )

    next_up: dict[str, list[str]] = {}
    for c in sorted(cards.values(), key=lambda c: (c["milestone"], c["id"])):
        if c["status"] == "ready":
            next_up.setdefault(c["suggested"], []).append(c["id"])

    out = {
        "generated_at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds"),
        "repo": args.repo,
        "milestones": milestones,
        "attention": attention,
        "next_up": next_up,
        "cards": [cards[cid] for cid in sorted(cards)],
    }
    Path(args.out).write_text(
        json.dumps(out, ensure_ascii=False, indent=1), encoding="utf-8"
    )
    if args.board:
        write_board(Path(args.board), out)
    counts: dict[str, int] = {}
    for c in cards.values():
        counts[c["status"]] = counts.get(c["status"], 0) + 1
    print(json.dumps(counts, ensure_ascii=False))


LABELS = {
    "done": "已完成",
    "review": "等审查",
    "in_progress": "进行中",
    "needs_you": "等你",
    "ready": "可开工",
    "blocked": "等依赖",
    "unverified": "已合并·待核验",
}


def write_board(path: Path, out: dict) -> None:
    """One flat row per card for the web board.

    The board derives milestones, the attention list and the totals from these
    rows itself, so a refresh replaces a single file.
    """
    names = {m["id"]: m["name"] for m in out["milestones"]}
    attention: dict[str, list[dict]] = {}
    for a in out["attention"]:
        attention.setdefault(a["card"], []).append(
            {"kind": a["kind"], "text": a["text"], "url": a.get("url", "")}
        )
    rows = []
    for c in out["cards"]:
        live = [p for p in c["prs"] if p["state"] == "open"] or c["prs"]
        pr = max(live, key=lambda p: p["number"]) if live else None
        rows.append(
            {
                "id": c["id"],
                "title": c["title"],
                "milestone": c["milestone"],
                "milestone_name": names.get(c["milestone"], ""),
                "difficulty": c["difficulty"],
                "suggested": c["suggested"],
                "you": c.get("you") or "",
                "status": c["status"],
                "status_label": c["status_label"],
                "waiting_on": " ".join(c["waiting_on"]),
                "pr": f"#{pr['number']}" if pr else "",
                "pr_url": pr["url"] if pr else "",
                "note": c.get("note", ""),
                "attention": attention.get(c["id"], []),
                "generated_at": out["generated_at"],
            }
        )
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(rows, ensure_ascii=False, indent=1), encoding="utf-8")


if __name__ == "__main__":
    main()
