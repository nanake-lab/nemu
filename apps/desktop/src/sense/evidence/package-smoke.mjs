// Reproducible ASAR/ABI and shutdown evidence. No live desktop sampling or settings writes.
import { createRequire } from 'node:module';
import { cpSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve, relative } from 'node:path';
import { pathToFileURL } from 'node:url';
import { execFileSync, spawn } from 'node:child_process';
import { createHash } from 'node:crypto';

const root = process.cwd();
const taskRoot = resolve(process.argv[2] ?? 'D:/Dev/cache/P6-08/package-smoke');
const evidence = resolve(process.argv[3] ?? 'reports/P6-08/evidence/package-smoke-01.json');
if (existsSync(evidence) || existsSync(taskRoot))
  throw new Error('Use new artifact/evidence paths');
mkdirSync(taskRoot, { recursive: true });
const require = createRequire(join(root, 'package.json'));
const viteRequire = createRequire(require.resolve('vite'));
const { build } = viteRequire('esbuild');
const { createPackageWithOptions } = await import(
  pathToFileURL('D:/Dev/cache/P6-08/tools/node_modules/@electron/asar/lib/asar.js').href
);
const electron = require('electron');
const sourceStage = join(taskRoot, 'app');
mkdirSync(sourceStage);
cpSync(join(root, 'apps/desktop/out'), join(sourceStage, 'out'), { recursive: true });
const copied = new Set();
function materialize(name, from) {
  if (copied.has(name)) return;
  const localRequire = createRequire(join(from, 'package.json'));
  let folder = dirname(localRequire.resolve(name));
  while (
    !existsSync(join(folder, 'package.json')) ||
    JSON.parse(readFileSync(join(folder, 'package.json'), 'utf8')).name !== name
  ) {
    const next = dirname(folder);
    if (next === folder) throw new Error(`Package root missing: ${name}`);
    folder = next;
  }
  const destination = join(sourceStage, 'node_modules', ...name.split('/'));
  mkdirSync(dirname(destination), { recursive: true });
  cpSync(folder, destination, {
    recursive: true,
    dereference: true,
    filter: (path) => !relative(folder, path).split(/[\\/]/).includes('node_modules'),
  });
  copied.add(name);
  const manifest = JSON.parse(readFileSync(join(folder, 'package.json'), 'utf8'));
  const packageRequire = createRequire(join(folder, 'package.json'));
  for (const dependency of Object.keys(manifest.dependencies ?? {}))
    materialize(dependency, folder);
  for (const dependency of Object.keys(manifest.optionalDependencies ?? {})) {
    try {
      packageRequire.resolve(dependency);
    } catch {
      continue;
    }
    materialize(dependency, folder);
  }
}
materialize('koffi', join(root, 'packages/desktop-sense'));
materialize('zod', join(root, 'apps/desktop'));
materialize('sherpa-onnx-node', join(root, 'apps/desktop'));
const baseline = execFileSync('git', ['show', '0b5412e:apps/desktop/src/main/index.ts'], {
  cwd: root,
  encoding: 'utf8',
});
await build({
  stdin: {
    contents: baseline,
    resolveDir: join(root, 'apps/desktop/src/main'),
    sourcefile: 'index.ts',
    loader: 'ts',
  },
  bundle: true,
  platform: 'node',
  target: 'node24',
  format: 'esm',
  packages: 'bundle',
  external: ['electron', 'sherpa-onnx-node'],
  outfile: join(sourceStage, 'baseline/main/index.js'),
});
// Preserve original main's runtime-relative renderer/utility paths.
cpSync(join(sourceStage, 'out/preload'), join(sourceStage, 'baseline/preload'), {
  recursive: true,
});
cpSync(join(sourceStage, 'out/renderer'), join(sourceStage, 'baseline/renderer'), {
  recursive: true,
});
cpSync(join(sourceStage, 'out/main/voice.js'), join(sourceStage, 'baseline/main/voice.js'));
cpSync(join(sourceStage, 'out/main/chunks'), join(sourceStage, 'baseline/main/chunks'), {
  recursive: true,
});
await build({
  stdin: {
    contents: `export { probeNativeRuntime, createSyntheticWin32Fixture } from './packages/desktop-sense/src/win32.ts';`,
    resolveDir: root,
    loader: 'ts',
  },
  bundle: true,
  platform: 'node',
  target: 'node24',
  format: 'esm',
  external: ['koffi'],
  outfile: join(sourceStage, 'native-probe.js'),
});
writeFileSync(
  join(sourceStage, 'package.json'),
  JSON.stringify({ name: 'nemu-p608-smoke', version: '0.0.0', type: 'module', main: 'boot.cjs' }),
);
writeFileSync(
  join(sourceStage, 'boot.cjs'),
  `
const { app, BrowserWindow } = require('electron');
const fs = require('node:fs');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
app.disableHardwareAcceleration();
// Avoid all user/system settings writes and never show a window.
app.setLoginItemSettings = () => {};
BrowserWindow.prototype.show = function () {};
let loaded = false;
const deadline = setTimeout(() => { process.stderr.write('startup deadline exceeded'); app.exit(2); }, 15000);
app.whenReady().then(async () => {
  if (process.env.P608_MODE === 'native') {
    const { probeNativeRuntime, createSyntheticWin32Fixture } = await import(pathToFileURL(path.join(__dirname, 'native-probe.js')).href);
    const runtime = probeNativeRuntime();
    const fixture = createSyntheticWin32Fixture(8);
    const before = performance.now();
    try { const stop = fixture.source.subscribe(() => {}); fixture.source.sample(); stop(); }
    finally { fixture.dispose(); }
    const cleanupMs = performance.now() - before;
    const binaries = runtime.binaries.map(p => { const diskPath = p.replace('app.asar' + path.sep, 'app.asar.unpacked' + path.sep); return { virtualPath: p, diskPath, exists: require('original-fs').existsSync(diskPath) }; });
    process.stdout.write('P608_JSON:' + JSON.stringify({ kind: 'native', isPackaged: app.isPackaged, runtime: { ...runtime, binaries }, cleanupMs }) + '\\n');
    clearTimeout(deadline); app.quit(); return;
  }
  await import(pathToFileURL(path.join(__dirname, process.env.P608_MODE === 'baseline' ? 'baseline/main/index.js' : 'out/main/index.js')).href);
  const ready = setInterval(() => {
    if (!app.__nemu_supervisor__) return;
    clearInterval(ready); clearTimeout(deadline); loaded = true;
    const children = app.getAppMetrics().map(p => p.pid).filter(pid => pid !== process.pid);
    setTimeout(() => { process.stdout.write('P608_JSON:' + JSON.stringify({ kind: 'shutdown', mode: process.env.P608_MODE, children }) + '\\n', () => app.quit()); }, 100);
  }, 10);
}).catch(error => { process.stderr.write(String(error.stack || error)); app.exit(3); });
`,
);
const runtimeDirectory = join(taskRoot, 'distribution');
cpSync(dirname(electron), runtimeDirectory, { recursive: true });
cpSync(electron, join(runtimeDirectory, 'Nemu-P608-Sense.exe'));
const archive = join(runtimeDirectory, 'resources', 'app.asar');
await createPackageWithOptions(sourceStage, archive, { unpackDir: 'node_modules' });
const results = [];
for (const mode of [
  'native',
  'baseline',
  'current',
  'current',
  'baseline',
  'baseline',
  'current',
]) {
  const scratch = join(taskRoot, 'tmp', `nemu-scaffold-${mode}-${results.length}`);
  mkdirSync(scratch, { recursive: true });
  const childEnv = {
    ...process.env,
    TMP: dirname(scratch),
    TEMP: dirname(scratch),
    NEMU_SCAFFOLD_USER_DATA: scratch,
    NEMU_DISABLE_SUPERVISOR: '1',
    P608_MODE: mode,
  };
  delete childEnv.ELECTRON_RUN_AS_NODE;
  const child = spawn(join(runtimeDirectory, 'Nemu-P608-Sense.exe'), [], {
    cwd: root,
    env: childEnv,
    stdio: ['ignore', 'pipe', 'pipe'],
    windowsHide: true,
  });
  let stdout = '',
    stderr = '',
    markerAt,
    marker;
  child.stdout.on('data', (data) => {
    stdout += data;
    for (const line of stdout.split(/\r?\n/))
      if (line.startsWith('P608_JSON:') && !marker) {
        marker = JSON.parse(line.slice(10));
        markerAt = performance.now();
      }
  });
  child.stderr.on('data', (data) => {
    stderr += data;
  });
  const outcome = await new Promise((resolve, reject) => {
    child.on('error', reject);
    child.on('exit', (code, signal) => resolve({ code, signal, exitedAt: performance.now() }));
  });
  await new Promise((resolve) => setTimeout(resolve, 2000));
  const survivors = [];
  // The direct child has exited; inspect only PIDs reported by that test application.
  for (const pid of marker?.children ?? []) {
    try {
      process.kill(pid, 0);
      survivors.push(pid);
    } catch {}
  }
  const result = {
    mode,
    ...outcome,
    marker,
    shutdownToExitMs: markerAt === undefined ? null : outcome.exitedAt - markerAt,
    survivors,
    stderr,
  };
  results.push(result);
  console.log(JSON.stringify(result));
  if (result.code !== 0 || !marker) break;
}
const hashes = Object.fromEntries(
  ['distribution/resources/app.asar', 'app/out/main/index.js', 'app/native-probe.js'].map(
    (file) => [
      file,
      createHash('sha256')
        .update(readFileSync(join(taskRoot, file)))
        .digest('hex'),
    ],
  ),
);
const result = {
  scope:
    'ASAR smoke distribution; full shell before/current with supervisor auto-start disabled, renderer windows suppressed, settings writes intercepted; synthetic sensing only',
  baseline: '0b5412e:apps/desktop/src/main/index.ts',
  dependencies: [...copied],
  hashes,
  results,
};
writeFileSync(evidence, JSON.stringify(result, null, 2), { flag: 'wx' });
if (results.some((r) => r.code !== 0 || !r.marker || r.survivors.length > 0)) process.exitCode = 1;
