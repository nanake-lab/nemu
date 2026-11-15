import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { sha } from './protocol-v2.mjs';

export function render(data) {
  if (data.pairs.length !== 22 || data.pairs.some((p, i) => p.position !== i + 1 || p.screened)) throw new Error('Expected 22 fixed visible pairs');
  const study = { ...data, materialHash: sha(JSON.stringify(data)) };
  const safe = JSON.stringify(study).replaceAll('<', '\\u003c').replaceAll('\u2028', '\\u2028').replaceAll('\u2029', '\\u2029');
  return readFileSync(new URL('./page.html', import.meta.url), 'utf8')
    .replace('/*__DATA__*/', `const study = ${safe};`)
    .replace('/*__APP__*/', readFileSync(new URL('./app.js', import.meta.url), 'utf8'));
}
if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  if (process.argv[2] !== '--readability-confirmed') throw new Error('Actual user readability confirmation required');
  const root = 'D:/Dev/cache/R-STEER-PAIR-02';
  const bytes = readFileSync(join(root, 'blind-02/blind-materials.json'));
  const commitment = JSON.parse(readFileSync(join(root, 'blind-02/commitment.json')));
  if (sha(bytes) !== commitment.blindMaterialsSha256) throw new Error('Frozen blind material changed');
  const data = { ...JSON.parse(bytes), version: 'R-STEER-PAIR-02-v2-no-deadline' };
  const html = render(data), out = join(root, 'delivery-v2-unlimited');
  mkdirSync(out);
  writeFileSync(join(out, 'comparison.html'), html, { flag: 'wx' });
  writeFileSync(join(out, '使用说明.txt'), '双击 comparison.html 打开。\n共22对，不限时，只记录实际用时；记得通读原文与知道条件对应关系分别勾选。\n完成后下载结果JSON并确认保存到本机。请把最终结果文件交回；页面不显示答案或分数。\n刷新或关闭会丢失未下载的选择，中断后不要重开一轮补答。\n', { flag: 'wx' });
  writeFileSync(join(root, 'page-commitment-v2-unlimited.json'), JSON.stringify({ ...commitment,
    page: 'created_after_user_readability_confirmation', pageVersion: data.version, responseDeadline: null, pageSha256: sha(html), materialHash: sha(JSON.stringify(data)),
    userReadabilityConfirmed: true, formalTrialsStartedByAgent: 0, judgments: 'not_run', analysis: 'not_run' }, null, 2) + '\n', { flag: 'wx' });
  console.log('Offline page built from exact frozen blind materials; no key read, no answers decrypted.');
}
