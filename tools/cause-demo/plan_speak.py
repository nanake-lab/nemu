from __future__ import annotations

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
VERIFIER_PROMPT = """你是核对员。下面每一条是 Nemu 要说的一句话，附有它引用的记录原文。逐条判断，只输出 JSON：
{"verdicts":[{"i":序号,"extra":true或false,"why":"一句理由"}]}
判断标准：
- 类型 past_event：这句话有没有说出所引记录里没有的事实？有，extra 为 true。
- 类型 past_self：这句话有没有说出所引记录里没有的过去举动或事件？只是给记录里的念头加情绪色彩、换说法，不算。有新的举动或事件，extra 为 true。
- 类型 now：这句话有没有断言过去某个时刻她的感受、想法或举动，而不只是说现在？有，extra 为 true。"""


def verify_points(
    raw: str,
    speakers: dict[str, str],
    *,
    self_quote: str | None = None,
    record_texts: dict[str, str] | None = None,
) -> dict[str, Any]:
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
        checks.append({"point": point, "kept": reason is None, "reason": reason})
        if reason is None:
            kept.append(point)
    if not kept and self_quote is None:
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


def join_points(points: list[dict[str, Any]], *, quoted_endings: bool = False) -> str:
    if not points:
        return PLAN2_FALLBACK
    texts: list[str] = [point["text"] for point in points]
    endings: tuple[str, ...] = ("。", "！", "？", "…")
    if quoted_endings:
        endings += tuple(punctuation + "」" for punctuation in endings)
    return "".join(
        text if text.rstrip().endswith(endings) else text + "。"
        for text in texts[:-1]
    ) + texts[-1]


def run_plan2(
    assembled: dict[str, Any],
    records: list[dict[str, Any]],
    api_key: str,
    *,
    self_quote: str = "loose",
    semantic_review: bool = False,
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
    result["verification"] = verification
    if semantic_review:
        return review_and_join(result, record_texts, api_key)
    return {**result, "answer": join_points(verification["kept"]), "valid": True}


def read_verdicts(raw: str, expected: set[int]) -> dict[int, dict[str, Any]]:
    parsed = json.loads(raw)
    if not isinstance(parsed, dict) or not isinstance(parsed.get("verdicts"), list):
        raise TypeError("expected a verdicts array")
    verdicts: dict[int, dict[str, Any]] = {}
    for item in parsed["verdicts"]:
        if not isinstance(item, dict):
            raise TypeError("invalid verdict")
        index = item.get("i")
        if (
            type(index) is not int or index not in expected or index in verdicts
            or type(item.get("extra")) is not bool or not isinstance(item.get("why"), str)
        ):
            raise ValueError("invalid or duplicate verdict")
        verdicts[index] = item
    if verdicts.keys() != expected:
        raise ValueError("missing verdict")
    return verdicts


def review_and_join(
    result: dict[str, Any], record_texts: dict[str, str], api_key: str
) -> dict[str, Any]:
    from chat import build_deepseek_body

    checks = result["verification"]["checks"]
    kept = result["verification"]["kept"]
    slow = any(point["type"] in ("past_self", "now") for point in kept)
    result.update(route="slow" if slow else "fast", verifier=None, semantic_checks=[])
    if not slow:
        return {**result, "answer": join_points(kept, quoted_endings=True), "valid": True}

    selected = {
        index: check["point"] for index, check in enumerate(checks, 1)
        if check["kept"] and check["point"]["type"] != "other"
    }
    blocks = []
    for index, point in selected.items():
        kind = point["type"]
        cited = "无" if kind == "now" else "\n".join(record_texts[cite] for cite in point["cites"])
        blocks.append(f"[{index}] 类型：{kind}\n话：{point['text']}\n引用记录：{cited}")
    body = build_deepseek_body([
        {"role": "system", "content": VERIFIER_PROMPT},
        {"role": "user", "content": "\n\n".join(blocks)},
    ], no_think=True)
    body.update(response_format={"type": "json_object"}, max_tokens=2048)
    verifier = run_call(body, api_key)
    result["verifier"] = verifier
    if not verifier["valid"]:
        return {**result, "error": "verifier_api_error"}
    try:
        verdicts = read_verdicts(verifier["response"]["text"], set(selected))
    except (TypeError, ValueError) as exc:
        return {**result, "error": f"invalid_verifier: {exc}"}
    result["semantic_checks"] = [
        {"i": index, "point": point, "extra": verdicts[index]["extra"], "why": verdicts[index]["why"],
         "reason": "verifier_extra" if verdicts[index]["extra"] else None}
        for index, point in selected.items()
    ]
    final = [
        check["point"] for index, check in enumerate(checks, 1)
        if check["kept"] and not verdicts.get(index, {}).get("extra")
    ]
    return {**result, "answer": join_points(final, quoted_endings=True), "valid": True}
