"""Render all informational dose observations locally, with escaped model text."""
from __future__ import annotations

import argparse
import html
import json
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("inputs", nargs="+", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    sections = []
    for source in args.inputs:
        result = json.loads(source.read_text(encoding="utf-8-sig"))
        title = html.escape(result["model"]["id"])
        sections.append(f"<h2>{title}</h2><p>{html.escape(str(result['generation_prompts']))}</p>")
        sections.append(f"<p>beta={html.escape(str(result.get('beta', 4)))}</p>")
        sections.append("<table><thead><tr><th>Language / seed</th><th>Coefficient</th>"
                        "<th>Output</th><th>Stop / seconds</th></tr></thead><tbody>")
        for sample in result.get("samples", []):
            sections.append(
                f"<tr><td>{html.escape(str(sample['language']))} / {sample['seed']}</td>"
                f"<td>ratio={html.escape(str(sample.get('ratio', 'legacy')))}; "
                f"a={sample['coefficient']}</td><td>{html.escape(sample['output'])}</td>"
                f"<td>{html.escape(sample['stop'])} / {sample['elapsed_s']:.3f}</td></tr>",
            )
        sections.append("</tbody></table>")
    document = """<!doctype html><html lang="zh"><meta charset="utf-8">
<title>P0-07 中性场景剂量比较</title><style>
body{font:16px/1.6 system-ui;margin:32px;max-width:1500px}table{border-collapse:collapse;width:100%}
td,th{border:1px solid #bbb;padding:10px;text-align:left;vertical-align:top}th{background:#eee}
td:nth-child(3){white-space:pre-wrap}tr:has(td:nth-child(2):is(:empty)){background:#eee}
</style><h1>P0-07 中性场景剂量比较</h1>
<p>信息性探索；各模型上方标明其探索预算，比例和系数逐条列出，不是生产参数或阈值批准。
同一模型内提示、方向、seed 与生成预算固定。全部输出保留，文本变化不等于方向或能力验证通过。</p>
""" + "\n".join(sections) + "</html>"
    with args.output.open("x", encoding="utf-8") as stream:
        stream.write(document)


if __name__ == "__main__":
    main()
