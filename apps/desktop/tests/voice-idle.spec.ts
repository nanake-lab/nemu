import { asrModelDirectory } from '@nemu/voice';
import { _electron as electron, expect, test } from '@playwright/test';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { existsSync } from 'node:fs';
import { assertQuiet } from '../../../packages/voice/src/asr/quiet.js';
test('P4-01 60-second utility CPU with live silent PCM', async () => {
  test.skip(
    process.env.NEMU_ASR_MEASURE !== '1',
    'Local quiet-window measurement only: pnpm measure:asr',
  );
  test.setTimeout(120000);
  assertQuiet();
  const directory = asrModelDirectory();
  test.skip(
    !existsSync(join(directory, 'model.int8.onnx')),
    'ASR models absent: CI never downloads models',
  );
  const scratch = await mkdtemp(join(tmpdir(), 'nemu-scaffold-asr-idle-'));
  const wav = Buffer.alloc(44 + 16000 * 2 * 70);
  wav.write('RIFF');
  wav.writeUInt32LE(wav.length - 8, 4);
  wav.write('WAVEfmt ', 8);
  wav.writeUInt32LE(16, 16);
  wav.writeUInt16LE(1, 20);
  wav.writeUInt16LE(1, 22);
  wav.writeUInt32LE(16000, 24);
  wav.writeUInt32LE(32000, 28);
  wav.writeUInt16LE(2, 32);
  wav.writeUInt16LE(16, 34);
  wav.write('data', 36);
  wav.writeUInt32LE(wav.length - 44, 40);
  const wavePath = join(scratch, 'silence.wav');
  await writeFile(wavePath, wav);
  const application = await electron.launch({
    args: [
      resolve('apps/desktop'),
      '--use-fake-device-for-media-stream',
      `--use-file-for-fake-audio-capture=${wavePath}`,
      '--use-gl=angle',
      '--use-angle=swiftshader',
      '--enable-unsafe-swiftshader',
    ],
    env: {
      ...process.env,
      NEMU_SCAFFOLD_USER_DATA: scratch,
      NEMU_DISABLE_SUPERVISOR: '1',
      NEMU_ASR_MODELS: directory,
    },
  });
  const samples: Array<{ at: number; cpuPercent: number }> = [];
  try {
    await expect
      .poll(() =>
        application.evaluate(({ app }) =>
          (app as unknown as { __nemu_voice__?: { isReady(): boolean } }).__nemu_voice__?.isReady(),
        ),
      )
      .toBe(true);
    await application.evaluate(({ app }) =>
      (
        app as unknown as { __nemu_channels__: { sendStageCommand(t: string, p: unknown): void } }
      ).__nemu_channels__.sendStageCommand('stage.mic.set', { enabled: true, mode: 'continuous' }),
    );
    const stage = application.windows().find((p) => p.url().includes('/stage/'));
    if (!stage) throw new Error('Stage absent');
    await expect
      .poll(() =>
        stage.evaluate(
          () =>
            (
              window as unknown as { nemuDesktop: { microphoneStatus(): { frames: number } } }
            ).nemuDesktop.microphoneStatus().frames,
        ),
      )
      .toBeGreaterThan(100);
    const start = performance.now();
    while (performance.now() - start < 60000) {
      assertQuiet(application.process().pid);
      const sample = await application.evaluate(
        ({ app }) => app.getAppMetrics().find((m) => m.name === 'Nemu Voice')?.cpu.percentCPUUsage,
      );
      if (sample === undefined) throw new Error('Voice utility metrics missing');
      samples.push({ at: performance.now() - start, cpuPercent: sample });
      await new Promise((r) => setTimeout(r, 1000));
    }
    const average = samples.reduce((s, v) => s + v.cpuPercent, 0) / samples.length;
    await writeFile(
      resolve('reports/P4-01/idle-cpu.json'),
      JSON.stringify(
        {
          measuredAt: new Date().toISOString(),
          reference:
            'Electron AppMetrics cpu.percentCPUUsage for Nemu Voice utility, real AudioWorklet streaming all-zero PCM at 50 packets/s',
          durationMs: performance.now() - start,
          average,
          max: Math.max(...samples.map((v) => v.cpuPercent)),
          threshold: 2,
          passed: average <= 2,
          samples,
        },
        null,
        2,
      ),
    );
    expect(average).toBeLessThanOrEqual(2);
  } finally {
    await application.close();
    await rm(scratch, { recursive: true, force: true });
  }
});
