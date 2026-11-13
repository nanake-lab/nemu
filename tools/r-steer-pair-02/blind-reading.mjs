import { randomBytes, createCipheriv } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { sha } from './protocol-v2.mjs';

// No page yet: only the full anonymous reading packet and a sealed future page assignment.
const root = 'D:/Dev/cache/R-STEER-PAIR-02';
const out = join(root, 'blind-02'), custody = join(root, 'custody-02');
mkdirSync(out); mkdirSync(custody);
const key = randomBytes(32), seed = randomBytes(32).toString('hex');
const seal = (bytes) => {
  const iv = randomBytes(12), cipher = createCipheriv('aes-256-gcm', key, iv);
  const body = Buffer.concat([cipher.update(bytes), cipher.final()]);
  return JSON.stringify({ algorithm: 'AES-256-GCM', iv: iv.toString('hex'), tag: cipher.getAuthTag().toString('hex'), body: body.toString('base64') }) + '\n';
};
writeFileSync(join(custody, 'key.bin'), key, { flag: 'wx', mode: 0o600 });
writeFileSync(join(custody, 'seed.sealed.json'), seal(Buffer.from(seed)), { flag: 'wx' });
// Randomization is fixed before reading any response text; custody is never uploaded.
const raw = readFileSync(join(root, 'v2-formal-02/result.json'));
const result = JSON.parse(raw);
if (result.status !== 'completed' || result.completed !== 44 || result.rows.length !== 44 || !result.guardVerified)
  throw new Error('No complete, guarded formal batch');
const materials = JSON.parse(readFileSync(join(root, 'v2-preparation-03/materials.json'))).formal;
const rank = (domain, id) => sha(JSON.stringify([seed, domain, id]));
const compare = (domain) => (a, b) => rank(domain, a.id).localeCompare(rank(domain, b.id)) || a.id.localeCompare(b.id);
for (const row of result.rows) {
  const bytes = readFileSync(join(root, 'v2-formal-02/requests', row.id, 'output.bin'));
  if (row.status !== 'completed' || !row.eog || row.capped || row.dose !== 30 || sha(bytes) !== row.sha256
      || new TextDecoder('utf-8', { fatal: true }).decode(bytes) !== row.text)
    throw new Error('Formal output binding differs');
}
const readingOrder = [...result.rows].sort(compare('read'));
const reading = '以下为全部44条回答原文，顺序已打乱。只需通读并确认能否理解，不用比较或猜测条件。\n\n'
  + readingOrder.map((row, i) => `【${String(i + 1).padStart(2, '0')}】\n${row.text}`).join('\n\n') + '\n';
const orderedMaterials = materials.map((row) => ({ ...row, id: `p${String(row.number).padStart(2, '0')}` })).sort(compare('order'));
const answerRows = [], pairs = [];
for (const [i, material] of orderedMaterials.entries()) {
  const rows = result.rows.filter((row) => row.material_id === material.id);
  if (rows.length !== 2 || new Set(rows.map((row) => row.sign)).size !== 2) throw new Error('Pairing differs');
  const positiveSide = parseInt(rank('side', material.id).slice(-1), 16) % 2 ? 'right' : 'left';
  const positive = rows.find((r) => r.sign === 'positive'), negative = rows.find((r) => r.sign === 'negative');
  const left = positiveSide === 'left' ? positive : negative, right = positiveSide === 'right' ? positive : negative;
  pairs.push({ position: i + 1, screened: false, context: material.text, left: left.text, right: right.text });
  answerRows.push({ position: i + 1, material_id: material.id, group: material.number <= 10 ? 'scene' : 'event',
    positiveSide, leftId: left.id, rightId: right.id, leftSha256: left.sha256, rightSha256: right.sha256 });
}
const blindedBytes = JSON.stringify({ version: 'R-STEER-PAIR-02-v2', question: '哪一句听起来更想亲近你？', pairs }) + '\n';
const answerBytes = Buffer.from(JSON.stringify({ version: 'R-STEER-PAIR-02-v2', seed, salt: randomBytes(32).toString('hex'),
  dose: .3, sourceSha256: sha(raw), readingOrder: readingOrder.map((row) => row.id), rows: answerRows }) + '\n');
const sealed = seal(answerBytes);
const commitment = { version: 'R-STEER-PAIR-02-v2', answers: 44, pairs: 22, readingSha256: sha(reading),
  blindMaterialsSha256: sha(blindedBytes), answerCommitmentSha256: sha(answerBytes), sealedFileSha256: sha(sealed),
  sourceResultSha256: sha(raw), page: 'not_created_pending_readability', humanJudgments: 'not_run', analysis: 'not_run' };
writeFileSync(join(out, '全部44条回答.txt'), reading, { flag: 'wx' });
writeFileSync(join(out, 'blind-materials.json'), blindedBytes, { flag: 'wx' });
writeFileSync(join(out, 'answers.sealed.json'), sealed, { flag: 'wx' });
writeFileSync(join(out, 'commitment.json'), JSON.stringify(commitment, null, 2) + '\n', { flag: 'wx' });
console.log('44 exact responses shuffled; 22 future assignments sealed; no page created, no answers decrypted.');
