import { _electron as electron, expect, test } from '@playwright/test';
import { mkdtemp, rm, access, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { existsSync, appendFileSync } from 'node:fs';
import { assertQuiet } from '../../../packages/voice/src/asr/quiet.js';
import type { ApplicationEvent } from '@nemu/contracts';
const modelDir =
  process.env.NEMU_ASR_MODELS ?? join(process.env.LOCALAPPDATA ?? '', 'Nemu/models/asr');
test('P4-01 utility ports, real AudioWorklet capture and disable', async () => {
  test.skip(
    !existsSync(join(modelDir, 'model.int8.onnx')),
    'ASR models absent: CI does not download models; run pnpm asr:models locally',
  );
  if (!process.env.CI) assertQuiet();
  const scratch = await mkdtemp(join(tmpdir(), 'nemu-scaffold-asr-e2e-'));
  const application = await electron.launch({
    args: [
      resolve('apps/desktop'),
      '--use-fake-device-for-media-stream',
      '--use-gl=angle',
      '--use-angle=swiftshader',
      '--enable-unsafe-swiftshader',
    ],
    env: {
      ...process.env,
      NEMU_SCAFFOLD_USER_DATA: scratch,
      NEMU_DISABLE_SUPERVISOR: '1',
      NEMU_ASR_MODELS: modelDir,
    },
  });
  application
    .process()
    .stderr?.on('data', (data: Buffer) =>
      appendFileSync(resolve('reports/P4-01/voice-electron-stderr.log'), data),
    );
  try {
    await expect
      .poll(() => application.windows().find((p) => p.url().includes('/stage/')))
      .toBeDefined();
    // Assert microphone/ASR errors; all avatar diagnostics remain in voice-stage-errors.json.
    await expect
      .poll(() =>
        application.evaluate(({ app }) => {
          const g = app as unknown as { __nemu_voice__?: { isReady(): boolean } };
          return g.__nemu_voice__?.isReady();
        }),
      )
      .toBe(true);
    await application.evaluate(({ app }) => {
      const g = app as unknown as {
        __nemu_channels__: { sendStageCommand(type: string, payload: unknown): void };
      };
      g.__nemu_channels__.sendStageCommand('stage.mic.set', { enabled: true, mode: 'pushToTalk' });
    });
    const stage = application.windows().find((p) => p.url().includes('/stage/'));
    if (!stage) throw new Error('Missing stage');
    await expect
      .poll(() =>
        application.evaluate(({ app }) => {
          const g = app as unknown as { __nemu_bus__: { getHistory(): ApplicationEvent[] } };
          return g.__nemu_bus__
            .getHistory()
            .filter((e) => e.type === 'stage.error' && /^(MIC_|ASR_)/u.test(e.payload.code))
            .map((e) => e.payload);
        }),
      )
      .not.toContainEqual({
        code: 'MIC_CAPTURE_FAILED',
        message: 'Microphone permission or device unavailable',
      });
    await expect
      .poll(() =>
        stage.evaluate(() => {
          const bridge = (
            window as unknown as {
              nemuDesktop: { microphoneStatus(): { active: boolean; frames: number } };
            }
          ).nemuDesktop;
          return bridge.microphoneStatus().frames;
        }),
      )
      .toBeGreaterThan(10);
    await application.evaluate(({ app }) => {
      const g = app as unknown as {
        __nemu_channels__: { sendStageCommand(type: string, payload: unknown): void };
      };
      g.__nemu_channels__.sendStageCommand('stage.mic.set', { enabled: false, mode: 'pushToTalk' });
    });
    await expect
      .poll(() =>
        application.evaluate(({ app }) => {
          const g = app as unknown as { __nemu_bus__: { getHistory(): ApplicationEvent[] } };
          return g.__nemu_bus__
            .getHistory()
            .filter((e) => e.type === 'stage.error' && /^(MIC_|ASR_)/u.test(e.payload.code))
            .map((e) => e.payload);
        }),
      )
      .toEqual([]);
    await expect
      .poll(() =>
        stage.evaluate(
          () =>
            (
              window as unknown as { nemuDesktop: { microphoneStatus(): { active: boolean } } }
            ).nemuDesktop.microphoneStatus().active,
        ),
      )
      .toBe(false);
    await access(join(scratch, 'logs'));
    expect(existsSync(join(scratch, 'core'))).toBe(false);
  } finally {
    const errors = await application.evaluate(({ app }) => (app as unknown as { __nemu_bus__?: { getHistory(): ApplicationEvent[] } }).__nemu_bus__?.getHistory().filter(e => e.type === 'stage.error'));
    await writeFile(resolve(process.env.NEMU_EVIDENCE_DIR ?? 'reports/P4-01', 'voice-stage-errors.json'), JSON.stringify(errors, null, 2));
    await application.close();
    await rm(scratch, { recursive: true, force: true });
  }
});
