import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { sha } from './study.mjs';

export const VERSION_V2 = 'R-STEER-PAIR-01-v2';
export function readPayload(html) {
  const match = html.match(/const study = (\{[^\r\n]+\});/u);
  assert(match, '缺少匿名题单');
  return JSON.parse(match[1]);
}

// 只修改匿名页面的版本与首页提示；不访问原库、随机源、密钥或答案。
export function revise(html) {
  const payload = readPayload(html);
  assert.equal(payload.version, 'R-STEER-PAIR-01-v1');
  const { materialHash, ...old } = payload;
  assert.equal(materialHash, sha(JSON.stringify(old)));
  const revised = { ...old, version: VERSION_V2 };
  revised.materialHash = sha(JSON.stringify(revised));
  const safe = JSON.stringify(revised).replaceAll('<', '\\u003c').replaceAll('\u2028', '\\u2028').replaceAll('\u2029', '\\u2029');
  const visible = payload.pairs.filter((pair) => !pair.screened).length;
  const consent = '我已确认本轮方案、问法和规则，可以开始正式作答。';
  assert(html.includes(consent));
  assert(html.includes('最多 12 对'));
  return html.replace(/const study = (\{[^\r\n]+\});/u, () => `const study = ${safe};`)
    .replace(consent, '我已批准第二版方案、问法和规则，可以开始正式作答。')
    .replace('最多 12 对', `第二版，共 ${visible} 对`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [sourcePage, outputDir] = process.argv.slice(2);
  assert(sourcePage && outputDir, '用法：revise-v2.mjs 第一版页面 新输出目录');
  const base = new URL('../../reports/R-STEER-PAIR-01/', import.meta.url);
  const v1 = JSON.parse(readFileSync(new URL('COMMITMENT.json', base), 'utf8'));
  const original = readFileSync(sourcePage);
  assert.equal(sha(original), v1.pageSha256, '必须使用已封存的第一版页面');
  const html = revise(original.toString('utf8'));
  const before = readPayload(original.toString('utf8')), after = readPayload(html);
  assert.deepEqual(after.pairs, before.pairs);
  assert.equal(after.question, before.question);
  assert.equal(after.pairs.filter((pair) => !pair.screened).length, 10);
  const commitment = {
    ...v1, version: VERSION_V2, answerVersion: v1.version,
    previousPageSha256: v1.pageSha256, pageSha256: sha(html),
    pairsSha256: sha(JSON.stringify(after.pairs)), materialHash: after.materialHash,
    preregistrationSha256: sha(readFileSync(new URL('PREREGISTRATION-V2.md', base))),
    minimumInjectedChoices: 9, familyGate: false,
  };
  mkdirSync(outputDir); // 不覆盖任何已冻结版本。
  writeFileSync(join(outputDir, 'comparison.html'), html);
  writeFileSync(join(outputDir, 'commitment.json'), JSON.stringify(commitment, null, 2) + '\n');
  process.stdout.write(JSON.stringify({ version: VERSION_V2, pairsUnchanged: true, visiblePairs: 10, answerOpened: false }) + '\n');
}
