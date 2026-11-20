import assert from 'node:assert/strict';
import { createDecipheriv } from 'node:crypto';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { sha, schedule } from './study.mjs';
import { readPayload, VERSION_V2 } from './revise-v2.mjs';

const keys = (object, expected) => assert.deepEqual(Object.keys(object).sort(), [...expected].sort());
const emptyCounts = () => ({ displayed: 0, injected: 0, zero: 0, tie: 0, quality: 0, unanswered: 0, screened: 0 });

// 预注册 v2 §4：固定10个显示对，至少9个注入；家族仅报告，质量与平局不删分母。
export function summarize(result, rows) {
  keys(result, ['version', 'materialHash', 'status', 'elapsedMs', 'answers']);
  assert.equal(result.version, VERSION_V2);
  assert(/^[a-f0-9]{64}$/u.test(result.materialHash));
  assert(['complete', 'timeout', 'stopped'].includes(result.status));
  assert(Number.isSafeInteger(result.elapsedMs) && result.elapsedMs >= 0);
  assert.equal(result.answers.length, 12);
  assert.equal(rows.length, 12);
  assert.equal(new Set(rows.map((row) => row.position)).size, 12);
  const names = [...new Set(rows.map((row) => row.family))].sort();
  assert.equal(names.length, 6);
  const families = names.map((_, i) => ({ family: `家族${i + 1}`, ...emptyCounts() }));
  const counts = emptyCounts(), trials = [];
  let recognized = 0;
  for (let i = 0; i < 12; i++) {
    const answer = result.answers[i], row = rows[i];
    keys(answer, ['position', 'quality', 'choice', 'recognized']);
    assert.equal(row.position, i + 1);
    assert.equal(answer.position, row.position);
    assert.equal(typeof answer.recognized, 'boolean');
    assert.equal(typeof row.screened, 'boolean');
    assert(['left', 'right'].includes(row.injectedSide));
    assert([null, 'left', 'right', 'tie'].includes(answer.choice));
    assert(['screened', 'unanswered', 'ok', 'left', 'right', 'both'].includes(answer.quality));
    let category;
    if (row.screened) {
      assert.equal(answer.quality, 'screened');
      assert.equal(answer.choice, null);
      assert.equal(answer.recognized, false);
      category = 'screened';
    } else if (['left', 'right', 'both'].includes(answer.quality)) {
      assert.equal(answer.choice, null);
      category = 'quality';
    } else {
      assert(['ok', 'unanswered'].includes(answer.quality));
      if (answer.quality === 'unanswered') assert.equal(answer.choice, null);
      category = answer.choice === null ? 'unanswered' : answer.choice === 'tie' ? 'tie'
        : answer.choice === row.injectedSide ? 'injected' : 'zero';
    }
    const family = families[names.indexOf(row.family)];
    counts[category]++; family[category]++;
    if (!row.screened) { counts.displayed++; family.displayed++; }
    if (answer.recognized) recognized++;
    trials.push({ position: row.position, family: family.family, category });
  }
  assert.equal(counts.displayed, 10); assert.equal(counts.screened, 2);
  assert.equal(counts.injected + counts.zero + counts.tie + counts.quality + counts.unanswered, 10);
  if (result.status === 'complete') assert.equal(counts.unanswered, 0);
  const valid = result.status === 'complete' && result.elapsedMs < 840000 && recognized === 0;
  return {
    version: VERSION_V2, counts, families, status: result.status, elapsedMs: result.elapsedMs,
    recognized, valid, threshold: 9, familyGate: false,
    criterionMet: valid && counts.injected >= 9,
    conclusion: !valid ? '本轮无效' : counts.injected >= 9
      ? '可感知信号成立（限本用户、本批材料与本轮规则）' : '本轮未达到预注册标准',
    trials,
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [publicRepo, evidenceRepo, custody] = process.argv.slice(2);
  assert(publicRepo && evidenceRepo && custody, '用法：analyze-v2.mjs 公开任务仓 私有证据仓 本机custody目录');
  const report = join(publicRepo, 'reports', 'R-STEER-PAIR-01');
  const base = join(evidenceRepo, 'R-STEER-PAIR-01');
  const post = join(base, 'post-answer-v2');
  const json = (path) => JSON.parse(readFileSync(path, 'utf8'));
  const commitment = json(join(report, 'COMMITMENT-V2.json'));
  const seal = json(join(post, 'result-seal-before-unblinding.json'));
  const resultBytes = readFileSync(join(post, 'result-original.json'));
  assert.equal(sha(resultBytes), seal.resultSha256);
  assert.equal(sha(readFileSync(join(post, 'observation-before-unblinding.json'))), seal.observationSha256);
  assert.equal(sha(readFileSync(join(base, 'v2', 'source', 'PREREGISTRATION-V2.md'))), commitment.preregistrationSha256);
  const pageBytes = readFileSync(join(base, 'v2', 'prepared', 'comparison.html'));
  assert.equal(sha(pageBytes), commitment.pageSha256);
  const payload = readPayload(pageBytes.toString('utf8'));
  const { materialHash, ...material } = payload;
  assert.equal(sha(JSON.stringify(material)), materialHash);
  assert.equal(sha(JSON.stringify(payload.pairs)), commitment.pairsSha256);
  const result = JSON.parse(resultBytes.toString('utf8'));
  assert.equal(result.materialHash, commitment.materialHash);
  assert.equal(result.materialHash, payload.materialHash);
  assert.equal(result.version, payload.version);
  const sealedBytes = readFileSync(join(base, 'prepared-v1', 'answers.sealed.json'));
  assert.equal(sha(sealedBytes), commitment.sealedFileSha256);
  const sealed = JSON.parse(sealedBytes.toString('utf8'));
  assert.equal(sealed.algorithm, 'AES-256-GCM');
  const output = join(post, 'analysis');
  mkdirSync(output); // 已存在就停止，不能静默覆盖本轮分析。
  const key = readFileSync(join(custody, 'key.bin'));
  const decipher = createDecipheriv('aes-256-gcm', key, Buffer.from(sealed.iv, 'hex'));
  decipher.setAuthTag(Buffer.from(sealed.tag, 'hex'));
  const plain = Buffer.concat([decipher.update(Buffer.from(sealed.body, 'base64')), decipher.final()]);
  key.fill(0);
  assert.equal(sha(plain), commitment.answerCommitmentSha256);
  const answer = JSON.parse(plain.toString('utf8'));
  assert.equal(answer.version, commitment.answerVersion);
  assert.equal(answer.source, commitment.sourceCommit);
  const draws = schedule(answer.rows.map((row) => ({ root: row.root, root_family: row.family })), answer.seed);
  for (const [i, row] of answer.rows.entries()) {
    for (const field of ['position', 'root', 'family', 'dose', 'injectedSide']) assert.equal(row[field], draws[i][field]);
    const pair = payload.pairs[i];
    assert.equal(pair.position, row.position); assert.equal(pair.screened, row.screened);
    if (row.screened) {
      assert.equal(pair.context, ''); assert.equal(pair.left, ''); assert.equal(pair.right, '');
    } else {
      assert.equal(sha(pair[row.injectedSide]), row.outputHashes[0]);
      assert.equal(sha(pair[row.injectedSide === 'left' ? 'right' : 'left']), row.outputHashes[1]);
    }
  }
  const summary = summarize(result, answer.rows);
  const { trials, ...aggregate } = summary;
  const evidence = {
    ...aggregate, resultSha256: seal.resultSha256, observationSha256: seal.observationSha256,
    answerCommitmentSha256: sha(plain), sealedFileSha256: sha(sealedBytes),
    answerCommitmentMatches: true, materialBindingMatches: true, originalResultUnmodified: true,
  };
  writeFileSync(join(output, 'answers-unsealed.json'), plain, { flag: 'wx' });
  writeFileSync(join(output, 'trial-classifications.json'), JSON.stringify(trials, null, 2) + '\n', { flag: 'wx' });
  writeFileSync(join(output, 'summary.json'), JSON.stringify(evidence, null, 2) + '\n', { flag: 'wx' });
  process.stdout.write(JSON.stringify(evidence, null, 2) + '\n');
}
