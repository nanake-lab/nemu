from __future__ import annotations

import json
from typing import Any

PLAN_PROMPT = """先不要回答。用 JSON 列出你准备在回答里说的每一个要点，格式：
{"points":[{"type":"past_event|past_self|now|other","text":"要点","cites":["记录编号"]}]}
type 含义：past_event＝过去发生过的事；past_self＝你过去某个时刻的想法、感受或举动；now＝你现在的想法或感受；other＝提问、招呼等其他内容。
past_event 和 past_self 必须在 cites 里写出依据的记录编号；now 和 other 的 cites 为空。"""
SPEAK_PROMPT = "把下面这些要点，用 Nemu 对竹说话的口吻说成一段自然的话。只说这些要点，不添加要点以外的过去的事，也不添加过去某个时刻的想法、感受或举动。要点里如果有「（没有可说的要点）」，就照实说你没有这方面的记录。"
FALLBACK = "（没有可说的要点）"


def verify_points(raw: str, speakers: dict[str, str]) -> dict[str, Any]:
    parsed = json.loads(raw)
    if not isinstance(parsed, dict) or not isinstance(parsed.get("points"), list):
        raise TypeError("expected an object with a points array")
    checks = []
    kept = []
    for point in parsed["points"]:
        reason = None
        if not isinstance(point, dict):
            reason = "invalid_point"
        elif point.get("type") not in ("past_event", "past_self", "now", "other"):
            reason = "unknown_type"
        elif not isinstance(point.get("text"), str) or not point["text"].strip():
            reason = "invalid_text"
        elif point["type"] in ("past_event", "past_self"):
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
        checks.append({"point": point, "kept": reason is None, "reason": reason})
        if reason is None:
            kept.append(point)
    if not kept:
        kept.append({"type": "other", "text": FALLBACK, "cites": []})
    return {"checks": checks, "kept": kept}


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
