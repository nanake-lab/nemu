/** Run only after explicit user quiet-window notice; compile this entry before running. */
import { writeFileSync } from 'node:fs';
import { cpus } from 'node:os';
import { createSyntheticWin32Fixture } from '../src/win32.js';
import { SenseEngine, startSenseLoop } from '../src/engine.js';

const mode = process.argv[2],
  output = process.argv[3];
if (
  !['control', 'sense'].includes(mode ?? '') ||
  !output ||
  process.env.NEMU_P608_QUIET_AUTHORIZED !== '1'
) {
  throw new Error('Requires a fresh output path and explicit user-authorized quiet window');
}
const fixture = createSyntheticWin32Fixture(32);
let samples = 0;
const source = {
  ...fixture.source,
  sample() {
    samples++;
    return fixture.source.sample();
  },
};
let stop: () => void;
try {
  if (mode === 'sense') {
    const engine = new SenseEngine(source, { idleThresholdMs: 60_000 }, { body() {}, bus() {} });
    stop = startSenseLoop(source, engine, () => {
      throw new Error('Measurement source fault');
    });
  } else {
    const timer = setInterval(() => {}, 1000);
    stop = () => clearInterval(timer);
  }
  try {
    await new Promise((resolve) => setTimeout(resolve, 30_000));
    const beforeSamples = samples,
      before = process.cpuUsage(),
      start = performance.now();
    await new Promise((resolve) => setTimeout(resolve, 60_000));
    const wallMs = performance.now() - start,
      cpu = process.cpuUsage(before);
    const oneCorePercent = (100 * (cpu.user + cpu.system)) / (wallMs * 1000);
    writeFileSync(
      output,
      JSON.stringify(
        {
          mode,
          pid: process.pid,
          warmupMs: 30_000,
          requestedSampleMs: 60_000,
          wallMs,
          cpu,
          logicalCpuCount: cpus().length,
          oneCorePercent,
          totalCapacityPercent: oneCorePercent / cpus().length,
          samples: samples - beforeSamples,
          fixtureWindows: fixture.frames.length,
          versions: process.versions,
        },
        null,
        2,
      ),
      { flag: 'wx' },
    );
  } finally {
    stop();
  }
} finally {
  fixture.dispose();
}
