#!/usr/bin/env python3
"""Derive every task card's status from GitHub.

Nobody edits card status by hand. The catalog (tasks/cards.json) holds static
facts; this module infers status from pull requests, branches and review
comments, so the board cannot go stale and parallel PRs never conflict on a
status file. The local board (serve_board.py) calls compute() on every refresh.

Usage (repository root, GitHub CLI authenticated):
    python tools/status/build_status.py [--out status.json] [--board board.json] [--repo owner/name]

Rules, first match wins:
    status_override in the catalog       -> that status
    open PR with label paused            -> paused (no user-action reminders)
    open PR with label needs-user        -> needs_you ("等你决定" while the PR is
                                            a draft, "等你合并" once it is ready)
    open draft PR                        -> in_progress (claimed)
    open ready PR                        -> review
    merged PR with label unverified      -> unverified (merged, review unfinished;
                                            dependants stay blocked)
    merged PR                            -> done
    branch task/<id>-* without any PR    -> in_progress (claimed)
    all dependencies done, card is yours -> needs_you
    all dependencies done                -> ready
    otherwise                            -> blocked

Review (AGENTS.md section 6, 2026-10-22): high cards, and any PR that touches a
guarded path (contracts, ledger storage, the action gate, executors, ADRs and
specs), need a cross-review comment whose first line is "审查结论：通过" or
"审查结论：有阻塞问题". Low and medium cards need none. The board asks the user
to forward such a PR to the reviewer, and holds back "等你合并" until it passed.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import re
import subprocess
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

REPO = "nanake-lab/nemu"
CARD = re.compile(r"\((P\d-\d{2})\)")
BRANCH = re.compile(r"^task/(P\d-\d{2})\b")
VERDICT = "审查结论："
# AGENTS.md section 6: whatever the difficulty, changes here need a cross review.
GUARDED = (
    "core/contracts/",
    "packages/contracts/",
    "core/crates/core-store/",
    "core/crates/action-gate/",
    "executors/",
    "docs/adr/",
    "docs/CONTRACTS.md",
    "docs/ARCHITECTURE.md",
    "docs/echo-core/",
)

LABELS = {
    "paused": "暂停",
    "done": "已完成",
    "review": "等审查",
    "in_progress": "进行中",
    "needs_you": "等你",
    "ready": "可开工",
    "blocked": "等依赖",
    "unverified": "已合并·待核验",
}


class GitHubError(RuntimeError):
    """The GitHub CLI is missing, not signed in, or a call failed."""


def gh_raw(args: list[str]) -> str:
    try:
        done = subprocess.run(
            ["gh", *args],
            capture_output=True,
            check=False,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
    except FileNotFoundError as exc:
        raise GitHubError(
            "没找到 gh 命令。先安装 GitHub CLI（winget install --id GitHub.cli），"
            "再运行 gh auth login 登录。"
        ) from exc
    if done.returncode != 0:
        raise GitHubError((done.stderr or done.stdout).strip() or "gh 调用失败")
    return done.stdout


def gh(path: str) -> list | dict:
    pages = json.loads(gh_raw(["api", "--paginate", "--slurp", path]))
    if pages and isinstance(pages[0], list):
        return [item for page in pages for item in page]
    return pages[0] if pages else {}


def load_catalog(repo: str, local: Path) -> dict:
    """The catalog on main, so the board is right whichever branch is checked out.

    Falls back to the local file (before the catalog reaches main, or offline).
    """
    try:
        raw = gh_raw(
            [
                "api",
                "-H",
                "Accept: application/vnd.github.raw",
                f"repos/{repo}/contents/tasks/cards.json?ref=main",
            ]
        )
        return json.loads(raw)
    except (GitHubError, json.JSONDecodeError):
        return json.loads(local.read_text(encoding="utf-8"))


def cards_of_pr(pr: dict) -> set[str]:
    found = set()
    m = BRANCH.match(pr["head"]["ref"])
    if m:
        found.add(m.group(1))
    found.update(CARD.findall(pr["title"]))
    return found


def windows_waiting(repo: str, sha: str) -> bool:
    """True when a Windows job for this commit waits for the self-hosted runner.

    The Windows job has its own workflow (windows.yml), so every run for the
    commit is checked, not only the newest one.
    """
    runs = gh(f"repos/{repo}/actions/runs?head_sha={sha}&per_page=10")
    runs = runs.get("workflow_runs", []) if isinstance(runs, dict) else []
    for run in runs:
        if run.get("status") == "completed":
            continue
        jobs = gh(f"repos/{repo}/actions/runs/{run['id']}/jobs")
        jobs = jobs.get("jobs", []) if isinstance(jobs, dict) else []
        if any(
            "Windows" in j["name"] and j["status"] in ("queued", "waiting")
            for j in jobs
        ):
            return True
    return False


def review_verdict(repo: str, number: int) -> str | None:
    """'pass', 'block' or None, from the latest review comment on the PR."""
    comments = gh(f"repos/{repo}/issues/{number}/comments?per_page=100")
    verdict = None
    for cm in comments if isinstance(comments, list) else []:
        body = (cm.get("body") or "").lstrip()
        if body.startswith(VERDICT):
            verdict = "pass" if body.startswith(VERDICT + "通过") else "block"
    return verdict


def touches_guarded(repo: str, number: int) -> bool:
    path = f"repos/{repo}/pulls/{number}/files?per_page=100"
    try:
        files = gh(path)
    except GitHubError:  # pagination refused: judge from the first page
        files = json.loads(gh_raw(["api", path]))
    return any(f["filename"].startswith(GUARDED) for f in files)


def open_pr_details(repo: str, pr: dict) -> dict:
    if any(label["name"] == "paused" for label in pr.get("labels", [])):
        return {}
    details = {"waiting_runner": windows_waiting(repo, pr["head"]["sha"])}
    if not pr.get("draft"):
        details["verdict"] = review_verdict(repo, pr["number"])
        details["guarded"] = touches_guarded(repo, pr["number"])
    return details


def needs_user_item(number: int, draft: bool, title: str = "") -> dict:
    """AGENTS.md section 6: a draft waits for a decision; a ready PR waits for the merge."""
    suffix = f"：{title}" if title else ""
    if draft:
        return {"kind": "decide", "text": f"PR #{number} 等你决定（草稿，还不能合并）{suffix}"}
    return {
        "kind": "merge",
        "text": f"PR #{number} 等你合并：先看 CI 全绿、没有落后 main{suffix}",
    }


def reviewer(card: dict | None, branch: str = "") -> str:
    """The other executor: by branch prefix for rule PRs, else by the card."""
    if branch.startswith("claude/"):
        return "Codex"
    if branch.startswith("codex/"):
        return "Claude"
    return "Codex" if card and card["suggested"] == "Claude" else "Claude"


def review_due(card: dict | None, pr: dict) -> bool:
    """A ready PR that needs a cross review and has not passed one yet."""
    if pr.get("draft"):
        return False
    needed = bool(pr.get("guarded")) or bool(card and card["difficulty"] == "高")
    return needed and pr.get("verdict") != "pass"


def review_item(number: int, who: str, why: str) -> dict:
    return {"kind": "cross_review", "text": f"PR #{number}（{why}）请转给 {who} 审查，通过前不要合并"}


def label_of(card: dict) -> str:
    if card["status"] != "review":
        return LABELS[card["status"]]
    live = [p for p in card["prs"] if p["state"] == "open" and not p["draft"]]
    if card["difficulty"] != "高" and not any(p.get("guarded") for p in live):
        return "等合并"
    verdicts = {
        p.get("verdict") for p in card["prs"] if p["state"] == "open" and not p["draft"]
    }
    if "pass" in verdicts:
        return "审查通过·待合并"
    if "block" in verdicts:
        return "审查有阻塞·待修改"
    return f"等 {reviewer(card, live[0]['branch'] if live else '')} 审查"


def compute(repo: str, catalog: dict) -> dict:
    cards = {c["id"]: dict(c) for c in catalog["cards"]}
    prs = gh(f"repos/{repo}/pulls?state=all&per_page=100")
    branches = [b["name"] for b in gh(f"repos/{repo}/branches?per_page=100")]

    relevant = [pr for pr in prs if cards_of_pr(pr) & cards.keys()]
    open_prs = [pr for pr in prs if pr["state"] == "open" and not pr.get("merged_at")]
    with ThreadPoolExecutor(max_workers=8) as pool:
        details = dict(
            zip(
                (pr["number"] for pr in open_prs),
                pool.map(lambda pr: open_pr_details(repo, pr), open_prs),
            )
        )

    for c in cards.values():
        c["prs"] = []
    for pr in relevant:
        state = "merged" if pr.get("merged_at") else pr["state"]
        if state == "closed":
            continue
        entry = {
            "number": pr["number"],
            "url": pr["html_url"],
            "title": pr["title"],
            "state": state,
            "draft": bool(pr.get("draft")),
            "branch": pr["head"]["ref"],
            "labels": [lbl["name"] for lbl in pr.get("labels", [])],
            "updated_at": pr["updated_at"],
            **details.get(pr["number"], {}),
        }
        for cid in cards_of_pr(pr) & cards.keys():
            cards[cid]["prs"].append(entry)
    claimed = {m.group(1) for b in branches if (m := BRANCH.match(b))}

    def direct(c: dict) -> str | None:
        if c.get("status_override"):
            return c["status_override"]["status"]
        live = [p for p in c["prs"] if p["state"] == "open"]
        if any("paused" in p["labels"] for p in live):
            return "paused"
        if any("needs-user" in p["labels"] for p in live):
            return "needs_you"
        if any(p["draft"] for p in live):
            return "in_progress"
        if live:
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
        for p in (p for p in c["prs"] if p["state"] == "open"):
            if "paused" in p["labels"]:
                continue
            item = {"card": c["id"], "pr": p["number"], "url": p["url"]}
            due = review_due(c, p)
            if "needs-user" in p["labels"] and not due:
                attention.append({**item, **needs_user_item(p["number"], p["draft"])})
            if p.get("waiting_runner"):
                attention.append(
                    {
                        **item,
                        "kind": "runner",
                        "text": f"PR #{p['number']} 的 Windows 检查在排队，需要开 runner",
                    }
                )
            if due and p.get("verdict") is None:
                why = f"{c['id']}，高难度" if c["difficulty"] == "高" else f"{c['id']}，改动了契约、权限或规格"
                attention.append({**item, **review_item(p["number"], reviewer(c, p["branch"]), why)})
        for p in c["prs"]:
            if p["state"] == "merged" and "unverified" in p["labels"]:
                attention.append(
                    {
                        "card": c["id"],
                        "pr": p["number"],
                        "url": p["url"],
                        "kind": "unverified",
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

    # Open PRs that belong to no card (rule changes, tooling) still need the
    # user when they wait for approval or for the Windows runner.
    other_attention = []
    for pr in open_prs:
        if any(label["name"] == "paused" for label in pr.get("labels", [])):
            continue
        if cards_of_pr(pr) & cards.keys():
            continue
        item = {"card": "", "pr": pr["number"], "url": pr["html_url"]}
        labels = [lbl["name"] for lbl in pr.get("labels", [])]
        info = {"draft": bool(pr.get("draft")), **details.get(pr["number"], {})}
        due = review_due(None, info)
        if "needs-user" in labels and not due:
            other_attention.append({**item, **needs_user_item(pr["number"], info["draft"], pr["title"])})
        if due and info.get("verdict") is None:
            who = reviewer(None, pr["head"]["ref"])
            other_attention.append({**item, **review_item(pr["number"], who, f"改动了契约、权限或规格：{pr['title']}")})
        if details.get(pr["number"], {}).get("waiting_runner"):
            other_attention.append(
                {
                    **item,
                    "kind": "runner",
                    "text": f"PR #{pr['number']} 的 Windows 检查在排队，需要开 runner",
                }
            )

    milestones = []
    for m in catalog["milestones"]:
        ids = [cid for cid, c in cards.items() if c["milestone"] == m["id"]]
        milestones.append(
            {
                **m,
                "total": len(ids),
                "done": sum(1 for cid in ids if cards[cid]["status"] == "done"),
                "remaining": sorted(
                    cid for cid in ids if cards[cid]["status"] != "done"
                ),
            }
        )

    next_up: dict[str, list[str]] = {}
    for c in sorted(cards.values(), key=lambda c: (c["milestone"], c["id"])):
        if c["status"] == "ready":
            next_up.setdefault(c["suggested"], []).append(c["id"])

    return {
        "generated_at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds"),
        "repo": repo,
        "milestones": milestones,
        "attention": attention,
        "other_attention": other_attention,
        "paused_prs": [
            {"number": pr["number"], "title": pr["title"], "url": pr["html_url"]}
            for pr in sorted(open_prs, key=lambda pr: pr["number"])
            if any(label["name"] == "paused" for label in pr.get("labels", []))
        ],
        "next_up": next_up,
        "cards": [cards[cid] for cid in sorted(cards)],
    }


def board_rows(out: dict) -> list[dict]:
    """One flat row per card. The board derives milestones, the attention list
    and the totals from these rows itself."""
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
    return rows


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default=REPO)
    ap.add_argument("--catalog", default="tasks/cards.json")
    ap.add_argument("--out", default="status.json")
    ap.add_argument(
        "--board", help="also write the board's row file (one row per card)"
    )
    args = ap.parse_args()

    out = compute(args.repo, load_catalog(args.repo, Path(args.catalog)))
    Path(args.out).write_text(
        json.dumps(out, ensure_ascii=False, indent=1), encoding="utf-8"
    )
    if args.board:
        path = Path(args.board)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(
            json.dumps(board_rows(out), ensure_ascii=False, indent=1), encoding="utf-8"
        )
    counts: dict[str, int] = {}
    for c in out["cards"]:
        counts[c["status"]] = counts.get(c["status"], 0) + 1
    print(json.dumps(counts))


if __name__ == "__main__":
    main()
