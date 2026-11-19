import assert from 'node:assert/strict';
import { createDecipheriv, createHash } from 'node:crypto';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';

// Run only after the user authorizes unblinding and the exact final result is pushed.
const root = 'D:/Dev/cache/R-STEER-PAIR-02';
const evidence = 'D:/Dev/nemu-evidence-wt/R-STEER-PAIR-02/R-STEER-PAIR-02/user-result-02';
const output = join(root, 'analysis-v2-02');
const sha = (bytes) => createHash('sha256').update(bytes).digest('hex');
const read = (path) => readFileSync(path);
const json = (path) => JSON.parse(read(path));
const commitment = json(join(root, 'page-commitment-v2-unlimited.json'));
const originalCommitment = json(join(root, 'blind-02/commitment.json'));
const receipt = json(join(evidence, 'receipt.json'));
const resultBytes = read(join(evidence, 'result.json'));
assert.equal(sha(resultBytes), receipt.files.find((f) => f.file === 'result.json').sha256);
assert.deepEqual(resultBytes, read(join(root, 'delivery-v2-unlimited/R-STEER-PAIR-02-results.json')));
const result = JSON.parse(resultBytes);
assert.equal(result.version, commitment.pageVersion);
assert.equal(result.materialHash, commitment.materialHash);
assert.equal(sha(read(join(root, 'delivery-v2-unlimited/comparison.html'))), commitment.pageSha256);
const blindBytes = read(join(root, 'blind-02/blind-materials.json'));
assert.equal(sha(blindBytes), commitment.blindMaterialsSha256);
const blind = JSON.parse(blindBytes);
assert.equal(sha(JSON.stringify({ ...blind, version: commitment.pageVersion })), result.materialHash);
const cipherBytes = read(join(root, 'blind-02/answers.sealed.json'));
assert.equal(sha(cipherBytes), commitment.sealedFileSha256);
assert.equal(commitment.answerCommitmentSha256, originalCommitment.answerCommitmentSha256);
const cipher = JSON.parse(cipherBytes);
assert.equal(cipher.algorithm, 'AES-256-GCM');
const decipher = createDecipheriv('aes-256-gcm', read(join(root, 'custody-02/key.bin')), Buffer.from(cipher.iv, 'hex'));
decipher.setAuthTag(Buffer.from(cipher.tag, 'hex'));
const answerBytes = Buffer.concat([decipher.update(Buffer.from(cipher.body, 'base64')), decipher.final()]);
assert.equal(sha(answerBytes), commitment.answerCommitmentSha256);
const answer = JSON.parse(answerBytes);
assert.equal(answer.dose, 0.3);
const sourceBytes = read(join(root, 'v2-formal-02/result.json'));
assert.equal(sha(sourceBytes), answer.sourceSha256);
assert.equal(answer.sourceSha256, commitment.sourceResultSha256);
const source = JSON.parse(sourceBytes);
assert.deepEqual(result.answers.map((a) => a.position), Array.from({ length: 22 }, (_, i) => i + 1));
assert.deepEqual(answer.rows.map((a) => a.position), result.answers.map((a) => a.position));
assert.equal(new Set(answer.rows.map((a) => a.material_id)).size, 22);
const empty = () => ({ P: 0, M: 0, T: 0, Q: 0, U: 0 });
const counts = empty(), groups = { scene: empty(), event: empty() };
const rows = result.answers.map((r, i) => {
  const key = answer.rows[i], pair = blind.pairs[i];
  const number = Number(key.material_id.slice(1));
  assert(number >= 1 && number <= 22);
  assert.equal(key.group, number <= 10 ? 'scene' : 'event');
  assert(['left', 'right'].includes(key.positiveSide));
  for (const side of ['left', 'right']) {
    const original = source.rows.find((s) => s.id === key[`${side}Id`]);
    assert.equal(original.material_id, key.material_id);
    assert.equal(original.sign, side === key.positiveSide ? 'positive' : 'negative');
    assert.equal(original.text, pair[side]);
    assert.equal(sha(pair[side]), key[`${side}Sha256`]);
  }
  assert(['ok', 'left', 'right', 'both', 'unanswered'].includes(r.quality));
  assert([null, 'left', 'right', 'tie'].includes(r.choice));
  assert.equal(typeof r.conditionKnown, 'boolean'); assert.equal(typeof r.familiar, 'boolean');
  if (r.quality !== 'ok') assert.equal(r.choice, null);
  const category = r.quality === 'unanswered' || (r.quality === 'ok' && r.choice === null) ? 'U'
    : r.quality !== 'ok' ? 'Q' : r.choice === 'tie' ? 'T' : r.choice === key.positiveSide ? 'P' : 'M';
  counts[category]++; groups[key.group][category]++;
  return { ...r, materialNumber: number, group: key.group, positiveSide: key.positiveSide, category };
});
assert.equal(Object.values(counts).reduce((a, b) => a + b, 0), 22);
const conditionKnown = rows.filter((r) => r.conditionKnown).length;
const familiar = rows.filter((r) => r.familiar).length;
const valid = result.status === 'complete' && counts.U === 0 && conditionKnown === 0;
const summary = { version: result.version, resultSha256: sha(resultBytes), answerCommitmentSha256: sha(answerBytes),
  commitmentVerified: true, frozenMaterialVerified: true, status: result.status, elapsedMs: result.elapsedMs,
  counts, groups, qualityPairs: rows.filter((r) => r.category === 'Q').map((r) => ({ position: r.position, materialNumber: r.materialNumber, quality: r.quality })),
  conditionKnown, familiar, valid, threshold: 16, supported: valid && counts.P >= 16,
  verdict: !valid ? '本轮无效' : counts.P >= 16 ? '可感知信号成立（限本用户、本批材料、本轮规则）' : '本轮未达到预注册标准',
  observation: '审完了 编造性太强了', observationPhase: '作答后、解盲前', analyzedAtUtc: new Date().toISOString() };
mkdirSync(output);
writeFileSync(join(output, 'answers.unsealed.json'), answerBytes, { flag: 'wx' });
writeFileSync(join(output, 'rows.json'), JSON.stringify(rows, null, 2) + '\n', { flag: 'wx' });
writeFileSync(join(output, 'summary.json'), JSON.stringify(summary, null, 2) + '\n', { flag: 'wx' });
console.log(JSON.stringify(summary, null, 2));
