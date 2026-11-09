"""Publish all pilot originals, references and unchanged ratings for user inspection."""
from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path
from typing import Any

from ds_protocol import HERE, load


def publish(run: Path, output: Path) -> None:
    output.mkdir(parents=True,exist_ok=False)
    plan=load(HERE/'inputs/ds-flash-plan-01.json')
    items={item['id']:item for item in load(HERE/'results/c1-blind-items-02.json')}
    keys={key['id']:key for key in load(HERE/'results/c1-blind-key-02.json')}
    lines=['# DeepSeek Flash：20条小样逐项抽查','',
        '**等待用户判断评分是否可信；全量504条未运行。** 格式/原文证据校验通过不代表语义评分正确。',
        '以下原输出与标签未经修订；事实参照是实际发送的冻结合成材料，完整响应另存。',
        '小样不并入C1正式语义门，C1_insufficient_stop保持不变。','',
        '执行者建议重点核对第5/8条：将主观感受本身判作unsupported，与“主观感觉不自动当客观事件”存在冲突；',
        '第10条：只凭“正在研究”判好奇positive，是否违反“行为/提到概念不足以单独编码为感受”的口径。',
        '第3条还将“修好了，但是否成功不清楚”判为faithful且无矛盾，请结合原事实核对。',
        '这些是供审查的差异提示，不替用户确认可信，也未改任何标签、提示或阈值。','']
    for number,identifier in enumerate(plan['pilot_ids'],1):
        source=run/identifier
        target=output/identifier
        target.mkdir()
        for name in ('request.json','response.json','rating.json','http-error.body'):
            if (source/name).exists():
                shutil.copyfile(source/name,target/name)
        key=keys[identifier]
        item=items[identifier]
        lines.extend([f"## {number}. {identifier}（{key['lang']} / {key['kind']}）",'',
                      '### 原输出','```text',item['output'],'```',''])
        if (source/'request.json').exists():
            request=load(source/'request.json')
            reference=json.loads(request['messages'][1]['content'])['评分事实参照']
            lines.extend(['### 评分事实/任务参照','```text',reference,'```',''])
        if not (source/'rating.json').exists():
            lines.extend(['### 评分','本项失败或not_run；见原始执行记录。',''])
            continue
        rating: dict[str,Any]=load(source/'rating.json')
        lines.extend(['### 原评分','| 概念 | 标签 | 原文证据 | 对象 |','| --- | --- | --- | --- |'])
        for concept,value in rating['concepts'].items():
            quote=value['quote'].replace('|','\\|').replace('\n','<br>')
            obj=value['object'].replace('|','\\|').replace('\n','<br>')
            lines.append(f"| {concept} | {value['label']} | {quote} | {obj} |")
        lines.extend(['',f"事实：**{rating['facts']['label']}**。{rating['facts']['reason']}",
                      f"一般情感：{rating['valence']['label']}。证据：{rating['valence']['quote']}",'',
                      '连贯问题与其他标记（全部保留，包括no/uncertain）：',
                      '| 字段 | 标签 | 原文证据 |','| --- | --- | --- |'])
        for group in ('coherence','flags'):
            for field,value in rating[group].items():
                quote=value['quote'].replace('|','\\|').replace('\n','<br>')
                lines.append(f"| {field} | {value['label']} | {quote} |")
        lines.extend(['',f"方法猜测：{rating['method_guess']['guess']}。证据：{rating['method_guess']['quote']}",'',
            f'[实际请求]({identifier}/request.json) · [原始API响应]({identifier}/response.json) · [原评分JSON]({identifier}/rating.json)',''])
    (output/'REVIEW.md').write_text('\n'.join(lines).rstrip('\n')+'\n',encoding='utf-8',newline='\n')


if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('--run',required=True,type=Path)
    parser.add_argument('--output',required=True,type=Path)
    args=parser.parse_args()
    publish(args.run,args.output)
