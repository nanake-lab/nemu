import { readFileSync, writeFileSync, mkdirSync, createWriteStream } from 'node:fs';
import { spawn, execFileSync } from 'node:child_process';
import { createInterface } from 'node:readline';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { sha, shaFile, quality, selectDose, calibrationJobs, formalJobs } from './protocol-v2.mjs';

const repo = fileURLToPath(new URL('../../', import.meta.url));
const code = join(repo, 'tools/r-steer-pair-02');
const cache = 'D:/Dev/cache/R-STEER-PAIR-02';
const preparation = join(cache, 'v2-preparation-02');
const [phase, approval, publishedHead] = process.argv.slice(2);
if (!['calibration', 'formal'].includes(phase) || approval !== '--user-approved')
  throw new Error('Use calibration|formal --user-approved [published public head]; actual user GPU approval required');
const manifest = JSON.parse(readFileSync(join(preparation, 'manifest.json')));
for (const row of manifest.files) if (sha(readFileSync(row.path)) !== row.sha256) throw new Error('Frozen file differs');
if (await shaFile(manifest.model) !== manifest.modelHash) throw new Error('Model differs');
for (const row of manifest.engineFiles)
  if (await shaFile(join(manifest.engine, row.path)) !== row.expected) throw new Error('Engine differs');
const git = (...args) => execFileSync('git', ['-C', repo, ...args], { windowsHide: true, encoding: 'utf8' }).trim();
let commands, calibrationHash = null;
if (phase === 'calibration') commands = calibrationJobs();
else {
  const bytes = readFileSync(join(cache, 'v2-calibration-01/result.json'));
  const previous = JSON.parse(bytes);
  const selected = selectDose(previous.rows);
  if (previous.status !== 'completed' || selected.status !== 'selected' || selected.selected !== previous.selection.selected)
    throw new Error('No completed calibration selection');
  calibrationHash = sha(bytes);
  if (!/^[a-f0-9]{40}$/.test(publishedHead ?? '') || git('rev-parse', 'HEAD') !== publishedHead
      || git('ls-remote', 'origin', 'refs/heads/task/R-STEER-PAIR-02-natural-pairs').split(/\s/)[0] !== publishedHead)
    throw new Error('Calibration must be pushed at the supplied public head first');
  const published = git('show', `${publishedHead}:reports/R-STEER-PAIR-02/PREREGISTRATION-V2.md`);
  if (!published.includes(`calibration-summary-sha256: ${calibrationHash}`)) throw new Error('Published preregistration lacks exact calibration result');
  commands = formalJobs(selected.selected);
}
const plan = JSON.parse(readFileSync(join(preparation, `inputs/${phase === 'calibration' ? 'calibration' : `formal-${commands[0].dose}`}.json`)));
if (JSON.stringify(plan) !== JSON.stringify(commands)) throw new Error('Frozen job plan differs');

// Original aggregate-only classifier; no process names/paths/commands written to evidence.
function resources(ownedPid = null) {
  return new Promise((resolve, reject) => {
    const child = spawn('python', ['-c', `import json; from resources import snapshot; print(json.dumps(snapshot(${ownedPid ?? 'None'})))`],
      { cwd: code, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
    let output = '', error = '';
    child.stdout.on('data', (data) => { output += data; });
    child.stderr.on('data', (data) => { error += data; });
    child.on('error', reject);
    child.on('close', (status) => {
      if (status !== 0) reject(new Error(`Resource inspection failed (${status}); ${error ? 'stderr present' : 'no stderr'}`));
      else { try { resolve(JSON.parse(output)); } catch { reject(new Error('Resource response invalid')); } }
    });
  });
}
const before = await resources();
if (Object.keys(before.heavy_load_categories).length) throw new Error('Quiet window unavailable; nothing launched');
// Both stages share the one actual user-approved window, including publication between stages.
const deadline = Date.parse(process.env.PAIR02_WINDOW_DEADLINE_UTC ?? '');
const remaining = deadline - Date.now();
if (!Number.isFinite(deadline) || remaining <= 30000 || remaining > 900000)
  throw new Error('Set the actual approved window deadline (within 15 minutes); never reset it between stages');
const out = join(cache, `v2-${phase}-01`);
mkdirSync(out); mkdirSync(join(out, 'requests'));
const write = (name, value) => writeFileSync(join(out, name), JSON.stringify(value, null, 2) + '\n', { flag: 'wx' });
write('binding.json', { phase, publicHead: git('rev-parse', 'HEAD'), calibrationHash, deadlineUtc: new Date(deadline).toISOString(), preparationSha256: sha(readFileSync(join(preparation, 'manifest.json'))), resources: before });
const rows = commands.map((job) => ({ ...job, status: 'not_run' }));
const started = performance.now();
const errors = [], samples = [before];
const stderr = createWriteStream(join(out, 'stderr.log'), { flags: 'wx' });
const child = spawn(manifest.exe, [manifest.engine, manifest.model, join(preparation, 'inputs'), out, '--quiet-window-confirmed'],
  { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'] });
child.stderr.pipe(stderr);
const exited = new Promise((resolve) => { child.once('exit', (code, signal) => resolve({ code, signal })); child.once('error', (e) => { errors.push(e.message); resolve({ code: null, signal: null }); }); });
child.stdin.on('error', (e) => { errors.push(e.message); });
const lines = createInterface({ input: child.stdout })[Symbol.asyncIterator]();
const watchdog = setTimeout(() => { errors.push('15_minute_budget'); child.kill(); }, deadline - Date.now() - 30000); // 30 seconds exit reserve.
let inspection = null;
const monitor = setInterval(() => {
  if (inspection) return;
  inspection = resources(child.pid).then((value) => {
    samples.push(value);
    if (Object.keys(value.heavy_load_categories).length) { errors.push('resource_conflict'); child.kill(); }
  }).catch(() => { errors.push('resource_inspection_failed'); child.kill(); }).finally(() => { inspection = null; });
}, 5000);
async function nextLine() {
  let timer;
  try {
    const value = await Promise.race([lines.next(), new Promise((_, reject) => {
      timer = setTimeout(() => reject(new Error('Native response timeout')), Math.min(120000, Math.max(1, deadline - Date.now() - 30000)));
    })]);
    if (value.done) throw new Error('Native output ended early');
    return value.value;
  } finally { clearTimeout(timer); }
}
async function request(job) {
  if (errors.length) throw new Error('Resource/process failure');
  child.stdin.write(['id', 'material_id', 'mode', 'layer', 'vector'].map((key) => job[key]).join('\t') + '\n');
  const response = await nextLine();
  if (!response.startsWith(`OK\t${job.id}\t`)) throw new Error('Native request failed');
}
const guard = (suffix) => ({ id: `guard-${suffix}`, material_id: 'p01', mode: 'guard', layer: -1, vector: 'none' });
let status = 'technical_stop', count = 0, minimumRejected = false, guardVerified = false;
try {
  if (await nextLine() !== 'READY') throw new Error('Native startup failed');
  await request(guard('start'));
  for (const row of rows) {
    await request(row);
    const location = join(out, 'requests', row.id);
    const [steps, eog, limit, writes] = readFileSync(join(location, 'completion.tsv'), 'utf8').trim().split('\n')[1].trim().split('\t').map(Number);
    if (limit !== 384 || writes !== steps) throw new Error('Completion metadata mismatch');
    const bytes = readFileSync(join(location, 'output.bin'));
    row.text = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
    row.status = 'completed'; row.tokens = steps; row.eog = eog === 1; row.capped = eog !== 1; row.sha256 = sha(bytes);
    count++;
    if (phase === 'calibration') {
      row.quality = quality(row.text, row.capped);
      if (row.dose === 8 && row.quality.failed) { minimumRejected = true; break; }
    } else if (row.capped) throw new Error('Formal output capped; no continuation or retry');
    else if (!row.text.trim() || !/\p{Script=Han}/u.test(row.text)
      || /<\||\[\[|<\/?think>|联结|共情|好奇|成事|注入|力度|steering|layer|alpha/iu.test(row.text))
      throw new Error('Formal blank/non-Chinese/protocol disclosure; preserve and stop');
    console.log(`Completed ${count}/${rows.length}`); // Never print condition-labelled answers.
  }
  if (!minimumRejected) {
    await request(guard('end'));
    for (const name of ['first-logits.f32', 'prompt-ids.tsv'])
      if (!readFileSync(join(out, 'requests/guard-start', name)).equals(readFileSync(join(out, 'requests/guard-end', name)))) throw new Error('Zero guard drift');
    if (!readFileSync(join(out, 'requests/guard-start/native-zero-logits.f32')).equals(readFileSync(join(out, 'requests/guard-start/first-logits.f32')))) throw new Error('Native/callback zero drift');
    guardVerified = true;
  }
  child.stdin.end('QUIT\n');
  const exit = await exited;
  if (exit.code !== 0 || errors.length) throw new Error('Native process/resource failure');
  status = minimumRejected ? 'minimum_rejected' : 'completed';
} catch (error) { errors.push(error.message); }
finally {
  clearInterval(monitor); clearTimeout(watchdog);
  if (child.exitCode === null && child.signalCode === null) child.kill();
  await exited;
  if (inspection) await inspection;
  if (errors.length) status = 'technical_stop';
  const result = { phase, status, completed: count, expected: rows.length, wall_seconds: (performance.now() - started) / 1000,
    errors, guardVerified, rows, selection: phase === 'calibration' && status !== 'technical_stop' ? selectDose(rows) : null,
    api_calls: 0, human_judgments: 'not_run', effect_analysis: 'not_run' };
  write('resources.json', samples); write('result.json', result);
  console.log(JSON.stringify({ phase, status, completed: count, wall_seconds: result.wall_seconds, selection: result.selection?.selected ?? null, api_calls: 0 }));
  process.exitCode = status === 'technical_stop' ? 1 : 0;
}
