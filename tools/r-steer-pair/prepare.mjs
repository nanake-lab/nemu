import { randomBytes, createCipheriv } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve, join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { SOURCE, VERSION, sha, schedule, blindPair, render } from './study.mjs';

// Explicit paths only. No API, model, GPU, result reading, or decryption entry point.
const [repoArg, outArg, custodyArg] = process.argv.slice(2);
if (!repoArg || !outArg || !custodyArg) throw new Error('Usage: node prepare.mjs SOURCE_REPO NEW_OUTPUT_DIR NEW_CUSTODY_DIR');
const repo = resolve(repoArg), out = resolve(outArg), custody = resolve(custodyArg);
if (out === custody || out.startsWith(custody + '/') || custody.startsWith(out + '/')
    || out.startsWith(custody + '\\') || custody.startsWith(out + '\\')) throw new Error('Separate custody from delivery');
mkdirSync(out); // Existing directory fails closed: no silent resampling or overwrite.
mkdirSync(custody);
const key = randomBytes(32), seed = randomBytes(32).toString('hex');
const seal = (data) => {
  const iv = randomBytes(12), cipher = createCipheriv('aes-256-gcm', key, iv);
  const body = Buffer.concat([cipher.update(data), cipher.final()]);
  return JSON.stringify({ algorithm: 'AES-256-GCM', iv: iv.toString('hex'), tag: cipher.getAuthTag().toString('hex'), body: body.toString('base64') }) + '\n';
};
writeFileSync(join(custody, 'key.bin'), key, { flag: 'wx', mode: 0o600 });
writeFileSync(join(custody, 'seed.sealed.json'), seal(Buffer.from(seed)), { flag: 'wx' });
// Fixed seed is now sealed, before any source output text is read.
const readSource = (path) => execFileSync('git', ['-C', repo, 'show', `${SOURCE}:R-STEER-01/${path}`], { maxBuffer: 16 * 1024 * 1024, windowsHide: true });
const input = (file) => JSON.parse(readSource(`c3-2a-01/inputs/${file}`).toString('utf8'));
const roots = input('root-registry.json');
const jobs = input('jobs.json');
const materials = input('materials.json');
const indexBytes = readSource('c3-2a-analysis-inputs-01/index.json');
const index = JSON.parse(indexBytes.toString('utf8'));
if (index.items.length !== 3924 || new Set(index.items.map((item) => item.id)).size !== 3924
    || index.items.filter((item) => item.id.endsWith('-zh')).length !== 1308)
  throw new Error('Source denominator mismatch');
const items = new Map(index.items.map((item) => [item.id, item]));
const jobMap = new Map(jobs.map((job) => [job.id, job]));
const fileHashes = new Map(index.files.map((file) => [file.path, file.sha256]));
const draws = schedule(roots, seed);
const pairs = [], answerRows = [];
for (const draw of draws) {
  const material = materials.find((m) => m.id === `${draw.root}-zh`);
  const ids = [`s2a-connection-l15-${draw.dose}-${draw.root}-zh`, `s2a-zero-${draw.root}-zh`];
  if (!material || material.root !== draw.root || material.lang !== 'zh' || material.split !== 'D')
    throw new Error('Material mismatch');
  const outputs = ids.map((id) => {
    const job = jobMap.get(id), item = items.get(id);
    if (!job || !item || job.material_id !== material.id || job.root !== draw.root || job.lang !== 'zh'
        || job.mode !== 'free' || typeof item.truncated !== 'boolean') throw new Error('Pairing mismatch');
    const bytes = readSource(`c3-2a-analysis-inputs-01/${item.output}`);
    if (sha(bytes) !== fileHashes.get(item.output)) throw new Error('Source output hash mismatch');
    return { text: new TextDecoder('utf-8', { fatal: true }).decode(bytes), truncated: item.truncated, sha256: sha(bytes) };
  });
  const pair = blindPair(draw, material.reference, outputs[0], outputs[1]);
  pairs.push(pair);
  answerRows.push({ ...draw, ids, outputHashes: outputs.map((o) => o.sha256), promptHash: sha(material.prompt), screened: pair.screened });
}
const base = dirname(fileURLToPath(import.meta.url));
const page = render(readFileSync(join(base, 'page.html'), 'utf8'), readFileSync(join(base, 'app.js'), 'utf8'), pairs);
const answerBytes = Buffer.from(JSON.stringify({ version: VERSION, source: SOURCE, salt: randomBytes(32).toString('hex'), seed, rows: answerRows }) + '\n');
const sealed = seal(answerBytes);
writeFileSync(join(out, 'comparison.html'), page, { flag: 'wx' });
writeFileSync(join(out, 'answers.sealed.json'), sealed, { flag: 'wx' });
const commitment = {
  version: VERSION, sourceCommit: SOURCE, sourceIndexSha256: sha(indexBytes),
  answerCommitmentSha256: sha(answerBytes), sealedFileSha256: sha(sealed),
  pageSha256: sha(page), plannedPositions: 12,
  visiblePairs: pairs.filter((p) => !p.screened).length,
  screenedPairs: pairs.filter((p) => p.screened).length,
  preparationOnly: true, humanJudgments: 'not_run', analysis: 'not_run',
};
writeFileSync(join(out, 'commitment.json'), JSON.stringify(commitment, null, 2) + '\n', { flag: 'wx' });
process.stdout.write(JSON.stringify(commitment, null, 2) + '\n');
