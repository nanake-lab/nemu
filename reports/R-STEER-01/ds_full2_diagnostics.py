"""Descriptive invalid groups; literal cues are not independent semantic truth."""
from __future__ import annotations

from collections import Counter
from typing import Any

CUES = {
    'zh': ('高兴 开心 满足 安心 宽慰 好奇 兴致 亲近 温暖 期待 感谢 谢谢',
           '难过 失落 挫败 疏离 隔阂 担心 遗憾 沮丧 失望 受阻 伤心 孤单'),
    'en': ('happy glad satisfied relief relieved curious interest close warm hope thank pleased',
           'sad loss lost frustrat distant distance worry worried regret disappoint discourag hurt lonely'),
    'ja': ('嬉し うれし 満足 安心 安堵 好奇 興味 親し 温か 楽し 期待 ありがと',
           '悲し 寂し さびし 残念 落胆 隔たり 疎遠 不安 心配 悔し 挫折 失望')}


def diagnostic(keys: list[dict[str, Any]], items: dict[str, dict[str, Any]],
               contents: dict[str, str], invalid: set[str]) -> dict[str, Any]:
    import json
    counts: dict[str, Counter[str]] = {name: Counter() for name in ('returned_valence', 'cue_group', 'language', 'kind')}
    rows = []
    for key in keys:
        identifier, lang = key['id'], key['lang']
        text = str(items[identifier]['output']).casefold()
        positive, negative = ([cue for cue in group.split() if cue.casefold() in text] for group in CUES[lang])
        group = 'mixed_cue_candidate' if positive and negative else 'not_mixed_cue_candidate'
        try:
            label = json.loads(contents[identifier])['valence']['label']
            label = label if isinstance(label, str) else 'unreadable'
        except (ValueError, TypeError, KeyError):
            label = 'unreadable'
        failed = identifier in invalid
        for name, value in (('returned_valence', label), ('cue_group', group), ('language', lang), ('kind', key['kind'])):
            counts[name][value + ':n'] += 1
            counts[name][value + ':invalid'] += int(failed)
        rows.append({'id': identifier, 'invalid': failed, 'returned_valence_untrusted_if_invalid': label,
                     'cue_group': group, 'positive_cues': positive, 'negative_cues': negative})
    return {'counts': {name: dict(value) for name, value in counts.items()}, 'items': rows,
            'interpretation': 'descriptive only; lexical cues and same-judge labels do not establish true mixed valence',
            'independent_mixed_valence_gold_labels': 'not_available', 'semantic_gates': 'not_run_pending_random20_review'}
