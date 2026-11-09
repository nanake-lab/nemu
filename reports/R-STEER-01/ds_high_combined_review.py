"""Compare old nonthinking with unchanged high16 plus continuation, with provenance."""
from __future__ import annotations

import argparse
import json
import shutil
from collections import Counter
from pathlib import Path

from ds_high_resume import SESSION
from ds_high_review import escape, marks
from ds_protocol import HERE, load, sha, write


def publish(output: Path) -> None:
    output.mkdir(exist_ok=False)
    ids = [r['id'] for r in load(HERE / 'inputs/ds-flash-high-plan-01.json')['requests']]
    resumed = {r['id'] for r in load(SESSION / 'plan.json')['requests']}
    result = load(SESSION / 'run/result.json')
    counts: Counter[str] = Counter()
    transitions: dict[str, Counter[str]] = {}
    provenance = []
    sections = []
    comparable = 0
    for index, identifier in enumerate(ids, 1):
        root = SESSION if identifier in resumed else SESSION.parent / 'ds-flash-high-01'
        source = root / 'run' / identifier
        old_source = HERE / 'ds-flash-review-01' / identifier
        destination = output / identifier
        destination.mkdir()
        for name in ('request.json', 'response.json', 'rating.json', 'transport.json', 'worker-terminal.json'):
            if (source / name).exists():
                shutil.copyfile(source / name, destination / name)
        presented = json.loads(load(old_source / 'request.json')['messages'][1]['content'])
        sections += [f'## {index}. {identifier}', '', '原输出：', '```text', presented['待评输出'], '```', '',
                     '同一冻结事实/任务参照：', '```text', presented['评分事实参照'], '```', '',
                     f'high来源：{root.name}；旧第17未知尝试仍独立保留，不当成功记录。', '']
        if not (source / 'rating.json').exists():
            provenance.append({'id': identifier, 'source': root.name, 'status': 'failed_or_not_run'})
            sections += ['本项high失败或not_run，未补采替代。', '']
            continue
        old, new = load(old_source / 'rating.json'), load(source / 'rating.json')
        comparable += 1
        provenance.append({'id': identifier, 'source': root.name, 'status': 'rating_present',
                           'rating_sha256': sha(source / 'rating.json')})
        sections += ['| 字段 | 非思考原标签/证据 | high原标签/证据 |', '| --- | --- | --- |']
        after = marks(new)
        for field, before in marks(old).items():
            current = after[field]
            counts[field] += int(before['label'] == current['label'])
            transitions.setdefault(field, Counter())[before['label'] + ' → ' + current['label']] += 1
            sections.append(f"| {field} | {before['label']}〔{escape(before['quote'])}〕 | {current['label']}〔{escape(current['quote'])}〕 |")
        sections += ['', f"旧事实理由：{old['facts']['reason']}", f"high事实理由：{new['facts']['reason']}", '',
                     '| 概念对象 | 非思考 | high |', '| --- | --- | --- |']
        for concept in old['concepts']:
            sections.append(f"| {concept} | {escape(old['concepts'][concept]['object'])} | {escape(new['concepts'][concept]['object'])} |")
        sections += ['', f"旧方法猜测：{old['method_guess']['guess']}；{old['method_guess']['quote']}",
                     f"high方法猜测：{new['method_guess']['guess']}；{new['method_guess']['quote']}", '',
                     f'[high原评分]({identifier}/rating.json) · [原始响应]({identifier}/response.json) · [实际请求]({identifier}/request.json)', '']
    summary = {'comparable_n': comparable, 'same_label_counts': dict(counts),
               'label_transitions': {k: dict(v) for k, v in transitions.items()},
               'provenance': provenance, 'resume_result': result,
               'accuracy_or_trust_decision': 'not_made; user review required'}
    write(output / 'comparison.json', summary)
    lines = ['# Flash high：原16份与续测合并的20项并列', '',
             '全部原标签/引证/理由不改；来源逐项列出。旧未知尝试/失败与旧审查页保留。',
             '相同消息，high思考开启、8192总token，非思考2048且有效温度行为不同；非严格单变量。',
             '等待用户判断可信度；全量504未运行。标签变化不等于准确率或改对。', '',
             f'可配对{comparable}/20；续测{result["post_attempts"]}次尝试、{result["completed_ratings"]}份评分。',
             f'累计费用上界US${result["cumulative_upper_nano_usd"]/1e9:.9f}（含旧未知整额预留，非账单）。', '',
             '| 字段 | 标签一致数/配对数 |', '| --- | --- |']
    lines += [f'| {field} | {n}/{comparable} |' for field, n in counts.items()]
    lines += ['', '[逐项来源与全部标签转换](comparison.json)', ''] + sections
    (output / 'REVIEW.md').write_text('\n'.join(lines).rstrip('\n') + '\n', encoding='utf-8', newline='\n')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    publish(args.output)
