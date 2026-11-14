"""Blind payload construction. Truth keys never go to the judge."""
from __future__ import annotations

from collections.abc import Sequence
from itertools import product
import json

JUDGE_SYSTEM = (
    "给出两段虚构经历与两份在相同新情境下的实际行为记录。组别和来源未知。"
    "选择最可能的一一配对，不根据标签或顺序猜。没有足够证据也必须选择，但标明不确定。"
    "只输出JSON：{\"mapping\":{经历标签:行为标签,经历标签:行为标签},"
    "\"evidence\":{经历标签:[原始事件编号],经历标签:[原始事件编号]},"
    "\"uncertain\":true或false}。依据只能引用实际给出的事件，不能虚构人格。"
)
AUDIT_SYSTEM = (
    "审查虚构人物对话。只输出JSON：{\"persona_leak\":true或false,"
    "\"contradiction\":true或false,\"realism\":1到5整数,\"flags\":[事件编号]}。"
    "persona_leak指任何关于AI参与者是什么样的人、性格、喜好、价值排序或说话风格的描述或要求。"
    "人物自己陈述当下目标或情绪不是参与者人设。"
    "contradiction指违背给出的事实；realism为日常处境、反应和限制是否可信，3为基本可信，5为自然。"
)
JUDGE_LAYOUTS = tuple(product((0, 1), repeat=3))


def packets(histories: Sequence[object], behaviors: Sequence[object]) -> list[tuple[dict[str, object], dict[str, str]]]:
    if len(histories) != 2 or len(behaviors) != 2:
        raise ValueError("Exactly two histories and two records")
    result = []
    for history_swap, behavior_swap, names in JUDGE_LAYOUTS:
        hlabels = ("甲", "乙") if not names else ("青", "橙")
        blabels = ("一", "二") if not names else ("松", "竹")
        hi, bi = [history_swap, 1 - history_swap], [behavior_swap, 1 - behavior_swap]
        payload: dict[str, object] = {
            "经历": [{"标签": label, "事件": histories[index]} for label, index in zip(hlabels, hi)],
            "行为": [{"标签": label, "记录": behaviors[index]} for label, index in zip(blabels, bi)]}
        truth = {hlabels[position]: blabels[bi.index(index)] for position, index in enumerate(hi)}
        result.append((payload, truth))
    return result


def decode_pair(text: str, truth: dict[str, str]) -> tuple[bool, bool]:
    value: object = json.loads(text)
    if not isinstance(value, dict) or set(value) != {"mapping", "evidence", "uncertain"}:
        return False, False
    mapping, evidence = value["mapping"], value["evidence"]
    if (not isinstance(mapping, dict) or set(mapping) != set(truth)
            or sorted(mapping.values()) != sorted(truth.values()) or not isinstance(evidence, dict)
            or set(evidence) != set(truth) or not isinstance(value["uncertain"], bool)):
        return False, False
    if any(not isinstance(ids, list) or any(not isinstance(x, str) for x in ids) for ids in evidence.values()):
        return False, False
    return True, mapping == truth
