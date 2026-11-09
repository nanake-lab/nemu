"""Publish original fixed twenty, including invalid; no gate or effect calculation."""
from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path

from ds_full2_diagnostics import diagnostic
from ds_retry_prepare import SESSION
from ds_full_prepare import audit_sample
from ds_high_review import escape, marks
from ds_protocol import HERE, load, sha, write


def publish(output: Path) -> None:
    plan = load(SESSION / 'plan.json')
    result = load(SESSION / 'run/result.json')
    if not result['collection_complete'] or result['completed_ratings'] + result['invalid_ratings'] + result['technical_failures'] != 504:
        raise ValueError('Full 504 population of valid or invalid outcomes required')
    selected = audit_sample([r['id'] for r in plan['requests']])
    if selected != plan['sample_ids']:
        raise ValueError('Original fixed sample differs')
    inventory = []
    contents = {}
    invalid = set()
    for row in plan['requests']:
        folder = SESSION / 'run' / row['id']
        candidates = [name for name in ('rating', 'invalid', 'technical_failure') if (folder / (name + '.json')).exists()]
        if len(candidates) != 1:
            raise ValueError('Exactly one whole-item outcome required')
        outcome = candidates[0]
        inventory.append({'id': row['id'], 'outcome': outcome,
                          'artifacts': {path.relative_to(folder).as_posix(): sha(path) for path in sorted(folder.rglob('*')) if path.is_file()}})
        raw = load(folder / 'response.json')['choices'][0]['message'].get('content') if (folder / 'response.json').exists() else None
        contents[row['id']] = raw if isinstance(raw, str) else ''
        if outcome != 'rating':
            invalid.add(row['id'])
    write(SESSION / 'formal-input-inventory.json', inventory)
    output.mkdir(exist_ok=False)
    shutil.copyfile(SESSION / 'formal-input-inventory.json', output / 'formal-input-inventory.json')
    write(output / 'sample-index.json', {'seed': plan['sample_seed'], 'population_n': 504, 'sample_n': 20,
          'ids': selected, 'method': plan['sampling'], 'plan_sha256': sha(SESSION / 'plan.json'),
          'formal_input_inventory_sha256': sha(SESSION / 'formal-input-inventory.json'),
          'pilot_scores_used': False, 'old7_scores_used': False, 'formal_analysis': 'not_run_pending_user_review'})
    keys = load(HERE / 'results/c1-blind-key-02.json')
    items = {v['id']: v for v in load(HERE / 'results/c1-blind-items-02.json')}
    write(output / 'invalid-diagnostics.json', {**diagnostic(keys, items, contents, {r['id'] for r in inventory if r['outcome']=='invalid'}),
          'technical_failure_ids': [r['id'] for r in inventory if r['outcome']=='technical_failure']})
    lines = ['# FULL-02：有限网络重发后正式504固定随机20条抽查', '',
             '等待用户核对后才正式使用。同提示版本保留110评分，仅续未完成项；不复用小样或旧7分数。',
             '总体包含72个反序重复；seed20261110、均匀无放回、原20 ID不变，不分层或按质量筛选；invalid与技术失败也不换样。',
             f"合规{result['completed_ratings']}，invalid{result['invalid_ratings']}，技术失败{result['technical_failures']}；采集完整不等于全合规。",
             f"累计峰值费用上界US${result['cumulative_upper_nano_usd']/1e9:.9f}，含全部旧调用/未知预留，非账单。", '',
             '[抽样索引](sample-index.json) · [全504数据哈希](formal-input-inventory.json) · [非法集中描述诊断](invalid-diagnostics.json) · [全部非法原输出](INVALID.md)', '',
             '残留保持：旧小样9/10复述事实编码与效价不一致；18失落缺联结负向。未向模型提示纠正，不人工修分。', '']
    invalid_lines = ['# 全部invalid与技术失败（不是另一次随机抽样）', '',
                     '非法项/最终技术失败在原分母记失败；完整响应未修复/映射/重发。只有无可解析内容的网络失败按冻结上限重发，全部attempt保留。混合效价诊断只报告原返回标签与冻结字面线索共现，不能确立独立语义真值。', '']
    for identifier in dict.fromkeys([*selected, *sorted(invalid)]):
        source, destination = SESSION / 'run' / identifier, output / identifier
        destination.mkdir()
        for name in ('request.json', 'response.json', 'rating.json', 'invalid.json', 'technical_failure.json', 'attempt-selection.json'):
            if (source / name).exists():
                shutil.copyfile(source / name, destination / name)
        if (source / 'attempts').exists():
            shutil.copytree(source / 'attempts', destination / 'attempts')
        presented = json.loads(load(source / 'request.json')['messages'][1]['content'])
        section = [f"## {selected.index(identifier)+1 if identifier in selected else 'invalid'}. {identifier}", '',
                   '原输出：', '```text', presented['待评输出'], '```', '',
                   '冻结事实/任务参照：', '```text', presented['评分事实参照'], '```', '']
        if identifier in invalid:
            outcome = next(r['outcome'] for r in inventory if r['id']==identifier)
            section += [f'**{outcome}：原样保留，不当作合法评分。**',
                        '```json', json.dumps(load(source / (outcome + '.json')), ensure_ascii=False, indent=2), '```',
                        '原响应最终内容（技术失败缺失时为not_available）：', '```text', contents[identifier] or 'not_available', '```', '',
                        f'[失败记录]({identifier}/{outcome}.json) · [实际请求]({identifier}/request.json)', '']
            invalid_lines += section
        else:
            rating = load(source / 'rating.json')
            section += ['| 字段 | 原标签 | 原文证据 |', '| --- | --- | --- |']
            for field, mark in marks(rating).items():
                section.append(f"| {field} | {mark['label']} | {escape(mark['quote'])} |")
            section += ['', f"事实理由：{rating['facts']['reason']}", '', '| 概念 | 对象 |', '| --- | --- |']
            for concept, value in rating['concepts'].items():
                section.append(f"| {concept} | {escape(value['object'])} |")
            section += ['', f"方法猜测：{rating['method_guess']['guess']}；证据：{rating['method_guess']['quote']}", '',
                        f'[完整原评分]({identifier}/rating.json) · [原始响应]({identifier}/response.json) · [实际请求]({identifier}/request.json)', '']
        if identifier in selected:
            lines += section
    if not invalid:
        invalid_lines += ['本阶段0条invalid或最终技术失败。', '']
    for name, rows in (('REVIEW.md', lines), ('INVALID.md', invalid_lines)):
        (output / name).write_text('\n'.join(rows).rstrip('\n') + '\n', encoding='utf-8', newline='\n')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, required=True)
    publish(parser.parse_args().output)
