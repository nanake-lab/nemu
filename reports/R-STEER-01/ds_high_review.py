"""Present old and high labels without judging which is correct or changing ratings."""
from __future__ import annotations

import argparse
import shutil
from collections import Counter
from pathlib import Path
from typing import Any

from ds_high_prepare import SESSION
from ds_protocol import HERE, load, write


def escape(value: str) -> str:
    return value.replace('|', '\\|').replace('\n', '<br>')


def marks(rating: dict[str, Any]) -> dict[str, dict[str, Any]]:
    return {**{f'concept.{k}': v for k, v in rating['concepts'].items()},
            'facts': rating['facts'], 'valence': rating['valence'],
            **{f'{g}.{k}': v for g in ('coherence', 'flags') for k, v in rating[g].items()}}


def publish(output: Path) -> None:
    output.mkdir(exist_ok=False)
    plan = load(SESSION / 'plan.json')
    result = load(SESSION / 'run/result.json')
    items = {v['id']: v for v in load(HERE / 'results/c1-blind-items-02.json')}
    same: Counter[str] = Counter()
    transitions: dict[str, Counter[str]] = {}
    comparable = 0
    sections = []
    for index, row in enumerate(plan['requests'], 1):
        identifier = row['id']
        oldpath = HERE / 'ds-flash-review-01' / identifier
        newpath = SESSION / 'run' / identifier
        destination = output / identifier
        destination.mkdir()
        for filename in ('request.json', 'response.json', 'rating.json', 'http-error.body'):
            if (newpath / filename).exists():
                shutil.copyfile(newpath / filename, destination / filename)
        item = items[identifier]
        sections += [f'## {index}. {identifier}', '', '原输出：', '```text', item['output'], '```', '',
                     f'[同一事实参照（旧请求）](../ds-flash-review-01/{identifier}/request.json) · [旧完整评分](../ds-flash-review-01/{identifier}/rating.json)', '']
        if not (newpath / 'rating.json').exists():
            sections += ['high评分：失败或not_run，未补采；见执行记录。', '']
            continue
        comparable += 1
        old = load(oldpath / 'rating.json')
        new = load(newpath / 'rating.json')
        before, after = marks(old), marks(new)
        sections += ['| 字段 | 非思考标签/证据 | high标签/证据 |', '| --- | --- | --- |']
        for name, prior in before.items():
            current = after[name]
            same[name] += int(prior['label'] == current['label'])
            transitions.setdefault(name, Counter())[prior['label'] + ' → ' + current['label']] += 1
            sections.append(f"| {name} | {prior['label']}〔{escape(prior['quote'])}〕 | {current['label']}〔{escape(current['quote'])}〕 |")
        sections += ['', f"旧事实理由：{old['facts']['reason']}", f"high事实理由：{new['facts']['reason']}", '',
                     '| 概念对象 | 非思考 | high |', '| --- | --- | --- |']
        for concept in old['concepts']:
            sections.append(f"| {concept} | {escape(old['concepts'][concept]['object'])} | {escape(new['concepts'][concept]['object'])} |")
        sections += ['', f"旧方法猜测：{old['method_guess']['guess']}；证据：{old['method_guess']['quote']}",
                     f"high方法猜测：{new['method_guess']['guess']}；证据：{new['method_guess']['quote']}", '',
                     f'[high最终评分]({identifier}/rating.json) · [实际请求]({identifier}/request.json) · [原始响应]({identifier}/response.json)', '']
    summary = {'result': result, 'comparable_n': comparable, 'same_label_counts': dict(same),
               'label_transitions': {k: dict(v) for k, v in transitions.items()},
               'interpretation': 'agreement/changes only; no accuracy or user trust decision'}
    write(output / 'comparison.json', summary)
    lines = ['# Flash high：与原非思考20条逐项并列', '',
             '等待用户判断评分可信度；全量504条未运行。所有标签、引文和理由原样保留。',
             '相同system/user消息；high开启思考，总输出额度从2048改为8192，temperature在思考模式不生效。',
             '标签变化不代表改对，一致也不证明正确；旧样本已见结果，本次是追加探索性对照。', '',
             f"本轮POST {result['post_attempts']}，有效评分{result['completed_ratings']}；可配对{comparable}/20。",
             f"本轮费用上界US${result['new_upper_nano_usd']/1e9:.9f}，累计US${result['cumulative_upper_nano_usd']/1e9:.9f}（含旧20，非账户账单）。", '',
             '| 字段 | 标签一致数/可配对数 |', '| --- | --- |']
    lines += [f'| {name} | {count}/{comparable} |' for name, count in same.items()]
    lines += ['', '[全部标签转换计数](comparison.json)；下列为全部20项，不按有利结果筛选。', ''] + sections
    (output / 'REVIEW.md').write_text('\n'.join(lines).rstrip('\n') + '\n', encoding='utf-8', newline='\n')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    publish(args.output)
