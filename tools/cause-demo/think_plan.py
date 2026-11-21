"""Number the delivered evidence and verify solitary thoughts before writeback."""
from __future__ import annotations

from collections import Counter
from typing import Any

from plan_speak import checked_points, join_points

THOUGHT_SPEAKER = "Nemu（独处时的念头）"
PROMPT = """用 JSON 按顺序列出这个念头的要点：
{"points":[{"type":"recall|now|guess|plan","text":"第一人称内心原话","cites":["记录编号"]}]}
recall 是已经发生的事或自己以前的想法，必须引用有依据的材料编号。引用以前的念头时，其回忆也要能追溯到外部记录。now 只写现在的感受或想法；guess 是明确的猜测或想象，写明「也许」「会不会」等；plan 是以后想做的事。这三种的 cites 都是空数组。要点依次连起来就是完整的念头。"""
# Explicit uncertainty or imagination in the Chinese thought itself (DEMO-THINK-05 §1).
GUESS_MARKERS = ("也许", "会不会", "可能", "或许", "是不是", "说不定", "大概", "兴许", "猜", "想象",
                 "没准", "难道", "莫非", "如果", "假如", "要是", "不知", "恐怕", "说不好", "搞不好")


def delivered_ids(binding: dict[str, Any]) -> set[str]:
    accepted = {rid for pack in binding["packs"] if pack["fate"] == "accepted"
                for rid in pack["records"]}
    return accepted | set(binding["base_records"])


class ThoughtPlan:
    def __init__(self, records: list[dict[str, Any]], binding: dict[str, Any]) -> None:
        # records is the time-filtered history, excluding the transient clock observation.
        self.history = {r["id"]: r for r in records}
        self.delivered = delivered_ids(binding) & self.history.keys()

    def instruction(self) -> dict[str, str]:
        records = [r for r in self.history.values() if r["id"] in self.delivered]
        locators = Counter((r["time"], r["speaker"]) for r in records)
        lines = [PROMPT, "本次材料中的编号："]
        for r in records:
            locator = (r["time"], r["speaker"])
            detail = f'：{r["text"]}' if locators[locator] > 1 else ""
            lines.append(f'{r["id"]} = [{r["time"]}] {r["speaker"]}{detail}')
        return {"role": "system", "content": "\n".join(lines)}

    def verify(self, sample: dict[str, Any]) -> dict[str, Any]:
        cache: dict[str, str | None] = {}

        def root_error(rid: str, visiting: set[str]) -> str | None:
            if rid in cache:
                return cache[rid]
            if rid in visiting:
                return "cyclic_cite"
            record = self.history.get(rid)
            if record is None:
                return "no_external_root"
            if record["speaker"] != THOUGHT_SPEAKER:
                return None
            visiting.add(rid)
            errors = {
                root_error(cite, visiting)
                for point in record.get("points", [])
                if isinstance(point, dict) and point.get("type") == "recall"
                and isinstance(point.get("cites"), list)
                for cite in point["cites"] if isinstance(cite, str)
            }
            visiting.remove(rid)
            error = None
            if "cyclic_cite" in errors:
                error = "cyclic_cite"
            elif None not in errors:
                error = "no_external_root"
            cache[rid] = error
            return error

        def check(point: dict[str, Any]) -> str | None:
            cites = point.get("cites")
            if not isinstance(cites, list) or not all(isinstance(c, str) for c in cites):
                return "invalid_cites"
            if point["type"] == "recall":
                if not cites:
                    return "empty_cites"
                if not self.delivered.issuperset(cites):
                    return "undelivered_cite"
                return next((err for c in cites if (err := root_error(c, set()))), None)
            if cites:
                return "unexpected_cites"
            if point["type"] == "guess" and not any(m in point["text"] for m in GUESS_MARKERS):
                return "unmarked_guess"
            return None

        raw = sample["text"]
        result = {**sample, "raw_text": raw, "text": "", "verification": None}
        if not sample["valid"]:
            return result
        try:
            verification = checked_points(raw, ("recall", "now", "guess", "plan"), check)
        except (TypeError, ValueError) as exc:
            return {**result, "valid": False, "error": f"invalid_plan: {exc}"}
        kept = verification["kept"]
        return {**result, "text": join_points(kept) if kept else "", "empty": not kept,
                "verification": verification}
