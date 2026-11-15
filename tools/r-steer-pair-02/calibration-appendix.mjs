import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';
import { sha, selectDose, CALIBRATION } from './protocol-v2.mjs';

// Writes a collapsed full appendix; never prints condition-labelled answers to chat.
const file = join(fileURLToPath(new URL('../../', import.meta.url)), 'reports/R-STEER-PAIR-02/PREREGISTRATION-V2.md');
const bytes = readFileSync('D:/Dev/cache/R-STEER-PAIR-02/v2-calibration-02/result.json');
const result = JSON.parse(bytes), original = readFileSync(file, 'utf8');
if (original.includes('## 9. 删除短句门后的定档结果')) throw new Error('Appendix already exists; do not overwrite');
const selection = result.status === 'technical_stop' ? { status: 'technical_stop', selected: null, groups: [] } : selectDose(result.rows);
if (result.status !== 'technical_stop' && JSON.stringify(selection) !== JSON.stringify(result.selection)) throw new Error('Selection mismatch');
const lines = ['','## 9. 删除短句门后的定档结果','',`calibration-summary-sha256: ${sha(bytes)}`,'',
  `执行状态：${result.status}；已执行${result.completed}/30；选档：${selection.selected === null ? '无，停止' : `±${selection.selected / 100}`}。模型API为0。`,'',
  '本节按预注册机械规则记录，不评价亲近感。正式盲测前不主动向用户展示下方带标签原文。','',
  '| 幅度 | 六条完整 | 六条全部通过 |','| --- | --- | --- |',
  ...selection.groups.map((g) => `| ±${g.dose / 100} | ${g.complete} | ${g.passed} |`),'',
  '<details>','<summary>完整定档回答与逐条筛查记录（带正负标签，正式盲测前请勿展开）</summary>',''];
for (const row of result.rows) {
  lines.push(`### ${row.dose / 100} · ${row.sign === 'positive' ? '正' : '负'} · ${CALIBRATION[Number(row.material_id.slice(1)) - 1]}`, '', `状态：${row.status}。`);
  if (row.status === 'completed') {
    if (row.reused_from) lines.push(`原文复用来源：${row.reused_from}，字节未变，仅按用户修订重新筛查。`);
    lines.push(`token步数：${row.tokens}；EOG：${row.eog}；cap：${row.capped}；screen：${row.quality?.failed ?? 'not_run'}；触发项：${row.quality?.reasons.join('、') || '无'}；原文SHA-256：${row.sha256}。`, '');
    const fence = '`'.repeat(Math.max(3, ...[...row.text.matchAll(/`+/g)].map((m) => m[0].length + 1)));
    lines.push(fence, row.text, fence, '');
  }
}
lines.push('</details>','',`技术错误记录：${JSON.stringify(result.errors)}。`, '');
writeFileSync(file, original + lines.join('\n'));
console.log(`Calibration appendix written; status=${result.status}; selected=${selection.selected === null ? 'none' : selection.selected / 100}. No answer text displayed.`);
