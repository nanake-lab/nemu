from __future__ import annotations

from collections.abc import Callable
import json
from typing import Any

PLAN_PROMPT = """先不要回答。用 JSON 列出你准备在回答里说的每一个要点，格式：
{"points":[{"type":"past_event|past_self|now|other","text":"要点","cites":["记录编号"]}]}
type 含义：past_event＝过去发生过的事；past_self＝你过去某个时刻的想法、感受或举动；now＝你现在的想法或感受；other＝提问、招呼等其他内容。
past_event 和 past_self 必须在 cites 里写出依据的记录编号；now 和 other 的 cites 为空。"""
SPEAK_PROMPT = "把下面这些要点，用 Nemu 对竹说话的口吻说成一段自然的话。只说这些要点，不添加要点以外的过去的事，也不添加过去某个时刻的想法、感受或举动。要点里如果有「（没有可说的要点）」，就照实说你没有这方面的记录。"
FALLBACK = "（没有可说的要点）"
PLAN2_PROMPT = """先不要回答。用 JSON 列出你这次要对竹说的话，按说出口的顺序，一个要点一句或几句，格式：
{"points":[{"type":"past_event|past_self|now|other","text":"你要直接说出口的原话","cites":["记录编号"]}]}
text 就是你对竹说的话本身，用第一人称，连起来读就是完整的回答。
type 含义：past_event＝过去发生过的事；past_self＝你过去某个时刻的想法、感受或举动；now＝你现在的想法或感受；other＝提问、招呼等其他内容。
past_event 和 past_self 必须在 cites 里写出依据的记录编号；now 和 other 的 cites 为空。"""
PLAN2_FALLBACK = "这件事我没有留下记录。"
PAST_MARKERS = ("那天", "当时", "那时", "那一刻", "那晚", "那会儿")
NOW_MARKERS = ("现在", "回头看", "回想", "想起来", "此刻")


def checked_points(
    raw: str, types: tuple[str, ...], check: Callable[[dict[str, Any]], str | None],
) -> dict[str, Any]:
    parsed = json.loads(raw)
    if not isinstance(parsed, dict) or not isinstance(parsed.get("points"), list):
        raise TypeError("expected an object with a points array")
    checks = []
    kept: list[dict[str, Any]] = []
    for point in parsed["points"]:
        reason: str | None
        if not isinstance(point, dict):
            reason = "invalid_point"
        elif point.get("type") not in types:
            reason = "unknown_type"
        elif not isinstance(point.get("text"), str) or not point["text"].strip():
            reason = "invalid_text"
        else:
            reason = check(point)
        checks.append({"point": point, "kept": reason is None, "reason": reason})
        if reason is None:
            kept.append(point)
    return {"checks": checks, "kept": kept}


def verify_points(
    raw: str,
    speakers: dict[str, str],
    *,
    self_quote: str | None = None,
    record_texts: dict[str, str] | None = None,
) -> dict[str, Any]:
    def check(point: dict[str, Any]) -> str | None:
        reason = None
        if point["type"] in ("past_event", "past_self"):
            cites = point.get("cites")
            if not isinstance(cites, list) or any(not isinstance(cite, str) for cite in cites):
                reason = "invalid_cites"
            elif not cites:
                reason = "empty_cites"
            elif any(cite not in speakers for cite in cites):
                reason = "undelivered_cite"
            elif point["type"] == "past_self" and not any(
                speakers[cite].startswith("Nemu") for cite in cites
            ):
                reason = "no_nemu_cite"
            elif self_quote == "strict" and point["type"] == "past_self":
                texts = record_texts or {}
                if not any(
                    speakers[cite].startswith("Nemu")
                    and texts.get(cite)
                    and texts[cite] in point["text"]
                    for cite in cites
                ):
                    reason = "not_verbatim"
        elif (
            self_quote is not None
            and point["type"] == "now"
            and any(marker in point["text"] for marker in PAST_MARKERS)
            and not any(marker in point["text"] for marker in NOW_MARKERS)
        ):
            reason = "now_refers_past"
        return reason

    result = checked_points(raw, ("past_event", "past_self", "now", "other"), check)
    if not result["kept"] and self_quote is None:
        result["kept"].append({"type": "other", "text": FALLBACK, "cites": []})
    return result


def run_call(body: dict[str, Any], api_key: str, *, echo: bool = False) -> dict[str, Any]:
    from chat import call_deepseek_stream, estimate_cost

    raw_stream: list[str] = []
    response = call_deepseek_stream(body, api_key, raw_stream=raw_stream, echo=echo)
    cost, period = estimate_cost(response["usage"], response["started_utc"])
    valid = bool(response["done"] and not response["error"] and response["text"].strip())
    return {
        "request": body,
        "response": response,
        "raw_stream": raw_stream,
        "valid": valid,
        "cost_usd": cost,
        "cost_period": period,
    }


def run_pipeline(
    assembled: dict[str, Any],
    records: list[dict[str, Any]],
    api_key: str,
    *,
    echo: bool = False,
) -> dict[str, Any]:
    from chat import build_deepseek_body

    delivered = set(assembled["delivered_ids"])
    speakers = {rec["id"]: rec["speaker"] for rec in records if rec["id"] in delivered}
    plan_body = build_deepseek_body(
        [*assembled["messages"], {"role": "system", "content": PLAN_PROMPT}], no_think=False
    )
    plan_body["response_format"] = {"type": "json_object"}
    plan = run_call(plan_body, api_key)
    result: dict[str, Any] = {"plan": plan, "verification": None, "speak": None, "valid": False}
    if not plan["valid"]:
        result["error"] = "plan_api_error"
        return result
    try:
        verification = verify_points(plan["response"]["text"], speakers)
    except (TypeError, ValueError) as exc:
        result["error"] = f"invalid_plan: {exc}"
        return result
    result["verification"] = verification
    speak_body = build_deepseek_body([
        {"role": "system", "content": SPEAK_PROMPT},
        {"role": "user", "content": "\n".join(
            " ".join(point["text"].splitlines()) for point in verification["kept"]
        )},
    ], no_think=True)
    speak = run_call(speak_body, api_key, echo=echo)
    result["speak"] = speak
    result["valid"] = speak["valid"]
    if not speak["valid"]:
        result["error"] = "speak_api_error"
    return result


def join_points(points: list[dict[str, Any]]) -> str:
    if not points:
        return PLAN2_FALLBACK
    texts: list[str] = [point["text"] for point in points]
    return "".join(
        text if text.rstrip().endswith(("。", "！", "？", "…")) else text + "。"
        for text in texts[:-1]
    ) + texts[-1]


def run_plan2(
    assembled: dict[str, Any],
    records: list[dict[str, Any]],
    api_key: str,
    *,
    self_quote: str = "loose",
) -> dict[str, Any]:
    from chat import build_deepseek_body

    delivered = set(assembled["delivered_ids"])
    speakers = {rec["id"]: rec["speaker"] for rec in records if rec["id"] in delivered}
    record_texts = {rec["id"]: rec["text"] for rec in records if rec["id"] in delivered}
    body = build_deepseek_body(
        [*assembled["messages"], {"role": "system", "content": PLAN2_PROMPT}], no_think=False
    )
    body["response_format"] = {"type": "json_object"}
    plan = run_call(body, api_key)
    result: dict[str, Any] = {
        "plan": plan, "verification": None, "answer": None, "valid": False, "self_quote": self_quote,
    }
    if not plan["valid"]:
        return {**result, "error": "plan_api_error"}
    try:
        verification = verify_points(
            plan["response"]["text"], speakers, self_quote=self_quote, record_texts=record_texts
        )
    except (TypeError, ValueError) as exc:
        return {**result, "error": f"invalid_plan: {exc}"}
    return {**result, "verification": verification, "answer": join_points(verification["kept"]), "valid": True}
