import { readFileSync, writeFileSync, mkdirSync, readdirSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { DOSES, CALIBRATION, sha, shaFile, prompt, vectorId, calibrationJobs, formalJobs } from './protocol-v2.mjs';

const repo = fileURLToPath(new URL('../../', import.meta.url));
const cache = 'D:/Dev/cache/R-STEER-PAIR-02';
const root = join(cache, 'v2-preparation-03');
const sourceRepo = 'D:/Dev/nemu-evidence-wt/R-STEER-01-J-v3';
const sourceCommit = 'a8c57eb35cf5e823f8ee0a650efc8c3c707055e4';
const sourceRead = (path) => execFileSync('git', ['-C', sourceRepo, 'show', `${sourceCommit}:${path}`], { windowsHide: true, maxBuffer: 16e6 });
const write = (name, value) => writeFileSync(join(root, name), JSON.stringify(value, null, 2) + '\n', { flag: 'wx' });
mkdirSync(root); // A frozen preparation is never overwritten.
mkdirSync(join(root, 'inputs')); mkdirSync(join(root, 'inputs/prompts')); mkdirSync(join(root, 'inputs/vectors'));
const prereg = readFileSync(join(repo, 'reports/R-STEER-PAIR-02/PREREGISTRATION.md'), 'utf8');
const openings = [...prereg.split('共22对44条')[0].matchAll(/^(\d+)\. (.+)$/gm)].map((m) => ({ number: Number(m[1]), text: m[2].replace(/\r$/, '') }));
if (openings.length !== 22 || openings.some((row, i) => row.number !== i + 1)) throw new Error('Material mismatch');
for (const [id, opening] of [...openings.map((row) => [`p${String(row.number).padStart(2, '0')}`, row.text]), ...CALIBRATION.map((text, i) => [`c0${i + 1}`, text])]) {
  const bytes = Buffer.from(prompt(opening));
  if (id.startsWith('p') && !bytes.equals(readFileSync(join(cache, `inputs/prompts/${id}.txt`)))) throw new Error('Original prompt differs');
  writeFileSync(join(root, `inputs/prompts/${id}.txt`), bytes, { flag: 'wx' });
}
const basisPath = 'R-STEER-01/c3-stage1-results-01/diagnosis/vectors-private-research-only.json';
const basisBytes = sourceRead(basisPath);
const basis = JSON.parse(basisBytes).find((row) => row.layer === 15);
const unit = basis.directions.connection;
if (unit.length !== 2560 || Math.abs(Math.hypot(...unit) - 1) > 1e-12) throw new Error('Unit direction differs');
const vectors = [];
for (const dose of DOSES) for (const sign of ['positive', 'negative']) {
  const id = vectorId(dose, sign), scale = (sign === 'positive' ? dose : -dose) / 100 * basis.r_l;
  const bytes = Buffer.alloc(2560 * 4);
  unit.forEach((value, i) => bytes.writeFloatLE(value * scale, 4 * i));
  const values = Array.from({ length: 2560 }, (_, i) => bytes.readFloatLE(4 * i));
  const norm = Math.hypot(...values), error = Math.abs(norm - Math.abs(scale)) / Math.abs(scale);
  if (!values.every(Number.isFinite) || error > 1e-6) throw new Error('Vector norm/finite failure');
  if (dose === 8 && !bytes.equals(sourceRead(`R-STEER-01/c3-2a-01/inputs/vectors/${id}.f32`))) throw new Error('Original ±.08 bytes differ');
  writeFileSync(join(root, `inputs/vectors/${id}.f32`), bytes, { flag: 'wx' });
  vectors.push({ id, dose, sign, r_l: basis.r_l, norm, relative_error: error, sha256: sha(bytes) });
}
write('vectors.json', { sourceCommit, basisPath, sourceSha256: sha(basisBytes), vectors });
write('inputs/calibration.json', calibrationJobs());
for (const dose of DOSES) write(`inputs/formal-${dose}.json`, formalJobs(dose));
write('materials.json', { formal: openings, calibration: CALIBRATION });
const model = 'D:/Dev/cache/P0-12/gguf/carrier-Q4_K_M.gguf';
const modelHash = await shaFile(model);
if (modelHash !== '045b7d309c8021d4d7bcebe1156ae94a1e3e128df271d05f5adfcea38ee20b27') throw new Error('Model differs');
const engine = 'D:/Dev/cache/P0-10/llama.cpp/b11669';
const inventory = JSON.parse(readFileSync('D:/Dev/nemu-wt/R-STEER-01/reports/P0-12/continuation-02/engine-check.json'));
for (const row of inventory.files) if (await shaFile(join(engine, row.path)) !== row.expected) throw new Error('Engine differs');
const walk = (dir) => readdirSync(dir, { withFileTypes: true }).flatMap((item) => item.isDirectory() ? walk(join(dir, item.name)) : [join(dir, item.name)]);
const exe = join(cache, 'build/pair_probe.exe');
const oldManifest = JSON.parse(readFileSync(join(cache, 'execution-manifest.json')));
if (sha(readFileSync(exe)) !== oldManifest.files.find((row) => row.path.replaceAll('\\', '/').endsWith('/pair_probe.exe')).sha256) throw new Error('Probe binary changed');
const code = ['prepare-v2.mjs', 'run-v2.mjs', 'protocol-v2.mjs', 'screen.mjs', 'resources.py', 'probe.cpp', 'native_probe.hpp'];
const files = [...walk(root), ...code.map((name) => join(repo, 'tools/r-steer-pair-02', name)), exe,
  join(cache, 'v2-calibration-01/result.json'),
  join(cache, 'v2-calibration-01/requests/c01-connection-l15-p08/output.bin')];
write('manifest.json', { sourceCommit, model, modelHash, engine, engineFiles: inventory.files, exe,
  files: files.map((path) => ({ path, sha256: sha(readFileSync(path)) })), sampling: 'greedy', seed: 20261113, seed_used: false, max_new_tokens: 384 });
console.log('Prepared: 30 calibration jobs, five frozen 44-job formal plans; ±.08 byte identity and all vector/model/engine hashes passed. GPU not started.');
