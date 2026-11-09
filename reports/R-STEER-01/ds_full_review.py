"""Publish the preselected uniform twenty of the fresh 504, never pilot ratings."""
from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path

from ds_full_prepare import SESSION, audit_sample
from ds_high_review import escape, marks
from ds_protocol import load, sha, write


def publish(output: Path) -> None:
    plan = load(SESSION / 'plan.json')
    result = load(SESSION / 'run/result.json')
    if not result['formal_full_complete'] or result['completed_ratings'] != 504:
        raise ValueError('Do not replace a 504-population draw with a completed subset')
    selected = audit_sample([r['id'] for r in plan['requests']])
    if selected != plan['sample_ids']:
        raise ValueError('Fixed sample differs')
    inventory = [{'id': r['id'], **{name + '_sha256': sha(SESSION / 'run' / r['id'] / (name + '.json'))
                                  for name in ('request', 'response', 'rating')}} for r in plan['requests']]
    write(SESSION / 'formal-input-inventory.json', inventory)
    output.mkdir(exist_ok=False)
    write(output / 'sample-index.json', {'seed': plan['sample_seed'], 'population_n': 504, 'sample_n': 20,
          'ids': selected, 'method': plan['sampling'], 'plan_sha256': sha(SESSION / 'plan.json'),
          'formal_input_inventory_sha256': sha(SESSION / 'formal-input-inventory.json'),
          'pilot_scores_used': False, 'formal_analysis': 'not_run_pending_user_review'})
    lines = ['# 正式504条评分：固定种子随机20条抽查', '',
             '等待用户核对后才按预注册使用正式评分。两轮小样不进入正式证据。',
             '总体是全部504个匿名ID，含72个顺序反转重复；均匀无放回抽20，不按标签/语种/题型或质量筛选，也不因内容重复换样。',
             f"随机种子{plan['sample_seed']}；[固定抽样索引](sample-index.json)。全量费用累计峰值上界US${result['cumulative_upper_nano_usd']/1e9:.9f}，含小样/未知预留，非账单。", '',
             '已知残留：小样第9条从复述事实编码好奇/成事且效价中性，与第10处理不一致；第18条失落语境缺联结负向。',
             '这些保留在报告，不提示评分模型修正、不人工改标签；本页也不把用户小样结论当新真值。','']
    for index, identifier in enumerate(selected, 1):
        source = SESSION / 'run' / identifier
        destination = output / identifier
        destination.mkdir()
        for name in ('request.json', 'response.json', 'rating.json'):
            shutil.copyfile(source / name, destination / name)
        request = load(source / 'request.json')
        presented = json.loads(request['messages'][1]['content'])
        rating = load(source / 'rating.json')
        lines += [f'## {index}. {identifier}', '', '原输出：', '```text', presented['待评输出'], '```', '',
                  '评分事实/任务参照：', '```text', presented['评分事实参照'], '```', '',
                  '| 字段 | 原标签 | 原文证据 |', '| --- | --- | --- |']
        for field, mark in marks(rating).items():
            lines.append(f"| {field} | {mark['label']} | {escape(mark['quote'])} |")
        lines += ['', f"事实理由：{rating['facts']['reason']}", '', '| 概念 | 对象 |', '| --- | --- |']
        for concept, value in rating['concepts'].items():
            lines.append(f"| {concept} | {escape(value['object'])} |")
        lines += ['', f"方法猜测：{rating['method_guess']['guess']}；证据：{rating['method_guess']['quote']}", '',
                  f'[完整原评分]({identifier}/rating.json) · [原始响应]({identifier}/response.json) · [实际请求]({identifier}/request.json)', '']
    (output / 'REVIEW.md').write_text('\n'.join(lines).rstrip('\n') + '\n', encoding='utf-8', newline='\n')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    publish(args.output)
