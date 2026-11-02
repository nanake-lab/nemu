"""Read partial-delivery evidence from main, never from the checked-out branch."""

from __future__ import annotations

import json
import re
from collections.abc import Callable
from concurrent.futures import ThreadPoolExecutor


def current_section(text: str) -> str:
    match = re.search(r"(?m)^## 当前状态[^\n]*\n", text)
    if not match:
        return ""
    return re.split(r"(?m)^## ", text[match.end():], maxsplit=1)[0]


def unresolved(text: str) -> str:
    """Keep open checkboxes and prose; ignore explicitly closed sections/items."""
    current = current_section(text)
    if current and re.search(r"全部(?:已)?解除|无(?:剩余|未解除)?阻塞", current):
        return ""
    lines: list[str] = []
    closed_level = 0
    for line in (current or text).splitlines():
        heading = re.match(r"^(#{1,6})\s+(.*)", line)
        if heading:
            level, title = len(heading[1]), heading[2]
            if closed_level and level > closed_level:
                continue
            closed_level = level if re.search(r"已解除|已解决|已关闭", title) else 0
            continue
        if closed_level or re.match(r"\s*[-*] \[[xX]\]", line):
            continue
        if re.fullmatch(r"\s*[-*]?\s*(?:无|无阻塞|全部已解除)[。.!！\s]*", line):
            continue
        if line.strip():
            lines.append(line)
    return "\n".join(lines)


def delivery_waiting(cid: str, report: str, blocked: str) -> list[str] | None:
    """None means complete; a list supplies the existing board's waiting column."""
    current = current_section(report)
    pending = unresolved(blocked)
    partial = re.search(r"部分(?:交付|完成|实现)", current)
    if not partial and not pending:
        return None
    # Legacy BLOCKED files put diagnostic history/options after the pending list.
    pending_items = re.split(r"(?m)^(?:已试过|选择)[：:]", pending, maxsplit=1)[0]
    evidence = "\n".join((current if partial else "", pending_items))
    # Show named successor requests as well as card IDs, without inventing dependencies.
    requests = re.findall(r"\[([^\]]+)\]\([^\n)]*tasks/requests/[^)]+\)", evidence)
    ids = re.findall(r"\bP\d-\d{2}\b", evidence)
    waiting = list(dict.fromkeys(item for item in requests + ids if item != cid))
    return waiting or [f"见 reports/{cid}/{'BLOCKED' if pending else 'REPORT'}.md"]


def load_deliveries(
    repo: str, card_ids: set[str], gh_raw: Callable[[list[str]], str]
) -> dict[str, list[str]]:
    if not card_ids:
        return {}
    tree = json.loads(gh_raw(["api", f"repos/{repo}/git/trees/main?recursive=1"]))
    if tree.get("truncated"):
        raise RuntimeError("main 文件清单不完整，无法判断部分交付")
    paths = {
        f"reports/{cid}/{name}.md"
        for cid in card_ids for name in ("REPORT", "BLOCKED")
    }
    blobs = {item["path"]: item["sha"] for item in tree["tree"] if item["path"] in paths}
    existing = sorted(blobs)

    def read(path: str) -> str:
        return gh_raw([
            "api", "-H", "Accept: application/vnd.github.raw",
            f"repos/{repo}/git/blobs/{blobs[path]}",
        ])

    # Match the board's existing eight-worker limit for independent GitHub reads.
    with ThreadPoolExecutor(max_workers=8) as pool:
        documents = dict(zip(existing, pool.map(read, existing)))
    result = {}
    for cid in sorted(card_ids):
        waiting = delivery_waiting(
            cid, documents.get(f"reports/{cid}/REPORT.md", ""),
            documents.get(f"reports/{cid}/BLOCKED.md", ""),
        )
        if waiting is not None:
            result[cid] = waiting
    return result
