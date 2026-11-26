// Run with the repository's tsx and a built nemu-core.exe.
import assert from 'node:assert/strict';
import { copyFile, mkdir, mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { SupervisedProcess } from '../../../../apps/desktop/src/supervisor/supervised-process.js';

const input = process.argv[2];
if (!input) throw new Error('Pass the built nemu-core.exe path');
const binary = resolve(input);
const previous = process.cwd();
const root = await mkdtemp(join(tmpdir(), 'nemu-host-desktop-'));
try {
  const executable = join(root, 'core/target/release/nemu-core.exe');
  await mkdir(dirname(executable), { recursive: true });
  await copyFile(binary, executable);
  process.chdir(root);
  const { REQUIRED_PROCESSES, createProcessConfigs } =
    await import('../../../../apps/desktop/src/supervisor/process-config.js');
  const spec = REQUIRED_PROCESSES.find((entry) => entry.name === 'nemu-core.exe');
  assert.ok(spec);
  const [config] = createProcessConfigs([spec]);
  assert.ok(config);
  assert.equal(config.command, executable);
  assert.equal(config.healthCheck?.type, 'stdout-heartbeat');
  const logs = join(root, 'logs');
  const host = new SupervisedProcess({ ...config, env: { LOCALAPPDATA: root } }, logs);
  try {
    await host.start();
    const started = host.getStatus();
    assert.ok(started.pid && !started.isPlaceholder);
    await delay(12_000);
    const status = host.getStatus();
    assert.equal(status.pid, started.pid);
    assert.equal(status.state, 'running');
    assert.equal(status.restartCount, 0);
    const log = await readFile(join(logs, spec.name + '.log'), 'utf8');
    const beats = log.split(/\r?\n/).filter(line => line === 'HEARTBEAT rehearsal waiting_for_initial_root');
    assert.ok(beats.length >= 3, 'missing real host heartbeats');
    console.log(`desktop supervisor: ${beats.length} heartbeats in 12 s, 0 restarts`);
  } finally {
    await host.stop();
  }
} finally {
  process.chdir(previous);
  await rm(root, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 });
}
