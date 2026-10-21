import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import type { ApplicationEvent } from '@nemu/contracts';
import { assertQuiet, assertQuietAsync } from './quiet.js';
import { cacheDirectory, verifyModels } from './models.js';
import { VoiceRuntime, type VoicePort } from './runtime.js';
import { loadTimingReferences } from './timing-reference.js';
import { replayPcm } from './replay.js';
const root =
  process.env.NEMU_ASR_DATA ?? resolve(process.env.LOCALAPPDATA ?? '', 'Nemu/asr-evaluation');
const attempt = `reports/P4-01/latency-attempt-${new Date().toISOString().replace(/[:.]/g, '-')}.json`;
class ReplayPort implements VoicePort {
  private handler: ((event: { data: unknown }) => void) | undefined;
  on(_type: 'message', handler: (event: { data: unknown }) => void): void {
    this.handler = handler;
  }
  start(): void {}
  close(): void {
    this.handler = undefined;
  }
  send(data: unknown): void {
    this.handler?.({ data });
  }
}
type Trace = { type: 'voice.vad' | 'voice.bargeIn'; state?: string; at: number };
const results: Array<{
  id: string;
  lang: string;
  sha256: string;
  sourceSha256: string;
  vadEndMs: number | null;
  bargeInMs: number | null;
  maximumLatenessMs: number;
  trace: Trace[];
}> = [];
let runtime: VoiceRuntime | undefined;
try {
  const references = await loadTimingReferences(root);
  assertQuiet();
  await verifyModels();
  let trace: Trace[] = [],
    collecting = false,
    pipelineFailure: string | undefined;
  runtime = new VoiceRuntime(cacheDirectory(), (event: ApplicationEvent) => {
    if (!collecting) return;
    if (event.type === 'voice.vad')
      trace.push({
        type: event.type,
        state: event.payload.state,
        at: event.ts - performance.timeOrigin,
      });
    if (event.type === 'voice.bargeIn')
      trace.push({ type: event.type, at: event.ts - performance.timeOrigin });
    if (event.type === 'stage.error') pipelineFailure = event.payload.code;
  });
  for (const ref of references) {
    assertQuiet();
    const audio = spawnSync(
      'ffmpeg',
      [
        '-v',
        'error',
        '-i',
        resolve(root, ref.path),
        '-ar',
        '16000',
        '-ac',
        '1',
        '-f',
        's16le',
        '-',
      ],
      { maxBuffer: 32 * 1024 * 1024 },
    );
    if (audio.status !== 0) throw new Error(`Audio decode failed: ${ref.id}`);
    const pcm = new Int16Array(
      audio.stdout.buffer.slice(
        audio.stdout.byteOffset,
        audio.stdout.byteOffset + audio.stdout.byteLength,
      ),
    );
    if (ref.speechEndMs > pcm.length / 16)
      throw new Error(`Reference exceeds audio duration: ${ref.id}`);
    const mic = new ReplayPort(),
      audioPort = new ReplayPort();
    runtime.attach(mic, audioPort);
    runtime.command({
      v: 2,
      id: ref.id,
      type: 'stage.mic.set',
      payload: { enabled: true, mode: 'continuous' },
    });
    audioPort.send({
      v: 2,
      id: ref.id,
      type: 'stage.speech.started',
      payload: { sentenceId: ref.id, at: 0 },
    });
    trace = [];
    pipelineFailure = undefined;
    collecting = true;
    let seq = 0;
    const active = runtime;
    const replay = await replayPcm(
      pcm,
      (block) => mic.send({ v: 2, seq: seq++, pcm: block }),
      assertQuietAsync,
    );
    collecting = false;
    const lastEnd = trace.filter((e) => e.type === 'voice.vad' && e.state === 'speechEnd').at(-1);
    const firstBarge = trace.find((e) => e.type === 'voice.bargeIn');
    results.push({
      id: ref.id,
      lang: ref.lang,
      sha256: ref.sha256,
      sourceSha256: ref.sourceSha256,
      vadEndMs: lastEnd ? lastEnd.at - replay.startedAt - ref.speechEndMs : null,
      bargeInMs: firstBarge ? firstBarge.at - replay.startedAt - ref.speechStartMs : null,
      maximumLatenessMs: replay.maximumLatenessMs,
      trace: trace.map((e) => ({ ...e, at: e.at - replay.startedAt })),
    });
    // Forced close cannot supply a missing speechEnd or barge-in for the measured trace.
    active.command({
      v: 2,
      id: ref.id,
      type: 'stage.mic.set',
      payload: { enabled: false, mode: 'continuous' },
    });
    audioPort.send({
      v: 2,
      id: ref.id,
      type: 'stage.speech.done',
      payload: { sentenceId: ref.id, interrupted: false, playedChars: 0 },
    });
    if (pipelineFailure) throw new Error(`Voice pipeline error: ${pipelineFailure}`);
  }
  const validEnds = results
    .flatMap((r) => (r.vadEndMs === null ? [] : [r.vadEndMs]))
    .sort((a, b) => a - b);
  const median = validEnds.length
    ? ((validEnds[Math.floor((validEnds.length - 1) / 2)] ?? 0) +
        (validEnds[Math.floor(validEnds.length / 2)] ?? 0)) /
      2
    : null;
  const passed =
    results.length === 60 &&
    results.every(
      (r) =>
        r.vadEndMs !== null &&
        r.vadEndMs >= 0 &&
        r.bargeInMs !== null &&
        r.bargeInMs >= 0 &&
        r.bargeInMs <= 250,
    ) &&
    median !== null &&
    median <= 300;
  const report = {
    measuredAt: new Date().toISOString(),
    status: passed ? 'PASS' : 'FAIL',
    method: 'P4-01-context-alignment-v3',
    referenceResolutionMs: 80,
    referenceUncertainty: 'Model estimates; 80 ms is timestamp grid, not a calibrated error bound',
    referenceSha256: createHash('sha256')
      .update(await readFile(resolve(root, 'timing-references.json')))
      .digest('hex'),
    medianVadEndMs: median,
    passed,
    nearThresholdIds: results
      .filter(
        (r) =>
          (r.vadEndMs !== null && Math.abs(r.vadEndMs - 300) <= 80) ||
          (r.bargeInMs !== null && Math.abs(r.bargeInMs - 250) <= 80),
      )
      .map((r) => r.id),
    perLanguage: (['zh', 'en', 'ja'] as const).map((lang) => {
      const rows = results.filter((r) => r.lang === lang);
      const ends = rows
        .flatMap((r) => (r.vadEndMs === null ? [] : [r.vadEndMs]))
        .sort((a, b) => a - b);
      return {
        lang,
        count: rows.length,
        medianVadEndMs: ends.length
          ? ((ends[Math.floor((ends.length - 1) / 2)] ?? 0) +
              (ends[Math.floor(ends.length / 2)] ?? 0)) /
            2
          : null,
        maximumVadEndMs: ends.length ? Math.max(...ends) : null,
        missingOrPremature: rows.filter(
          (r) => r.vadEndMs === null || r.vadEndMs < 0 || r.bargeInMs === null || r.bargeInMs < 0,
        ).length,
      };
    }),
    results,
  };
  await writeFile(attempt, JSON.stringify(report, null, 2), { flag: 'wx' });
  await writeFile('reports/P4-01/latency.json', JSON.stringify(report, null, 2));
  if (!passed) process.exitCode = 1;
} catch (error) {
  const status = results.length ? 'INVALID' : 'BLOCKED';
  const report = {
    status,
    passed: false,
    reason: error instanceof Error ? error.message : 'Unknown measurement failure',
    results,
  };
  await writeFile(attempt, JSON.stringify(report, null, 2), { flag: 'wx' });
  await writeFile('reports/P4-01/latency.json', JSON.stringify(report, null, 2));
  throw error;
} finally {
  runtime?.close();
}
