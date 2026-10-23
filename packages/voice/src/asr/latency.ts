import { asrDataDirectory } from './cache.js';
import { writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import type { ApplicationEvent } from '@nemu/contracts';
import { assertQuiet, assertQuietAsync } from './quiet.js';
import { cacheDirectory, verifyModels } from './models.js';
import { VoiceRuntime, type VoicePort } from './runtime.js';
import { loadHybridReferences } from './hybrid-reference.js';
import { sampleStats } from './measurement-stats.js';
import { assessLatency } from './latency-acceptance.js';
import { replayPcm, isFatalVoiceDiagnostic } from './replay.js';
const root = asrDataDirectory();
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
type Trace = {
  type: 'voice.vad' | 'voice.bargeIn';
  state?: string;
  at: number;
  stampToCallbackMs: number;
  feedToStampMs: number | null;
};
const results: Array<{
  id: string;
  lang: string;
  sha256: string;
  sourceSha256: string;
  vadEndMs: number | null;
  bargeInMs: number | null;
  maximumLatenessMs: number;
  trace: Trace[];
  diagnosticCodes: string[];
  vadEndReceiptMs: number | null;
  bargeInReceiptMs: number | null;
  processing: {
    feedCallWallMs: ReturnType<typeof sampleStats>;
    eventStampToLocalCallbackMs: ReturnType<typeof sampleStats>;
    feedStartToEventStampMs: ReturnType<typeof sampleStats>;
    isolatedNativeVadMs: null;
    externalIpcSendMs: null;
  };
}> = [];
let runtime: VoiceRuntime | undefined;
try {
  const bundle = await loadHybridReferences(root);
  const references = bundle.references;
  assertQuiet();
  await verifyModels();
  let trace: Trace[] = [],
    diagnosticCodes: string[] = [],
    collecting = false,
    currentFeedStart: number | undefined,
    pipelineFailure: string | undefined;
  runtime = new VoiceRuntime(cacheDirectory(), (event: ApplicationEvent) => {
    if (!collecting) return;
    const at = event.ts - performance.timeOrigin;
    const timing = {
      at,
      stampToCallbackMs: performance.now() - at,
      feedToStampMs: currentFeedStart === undefined ? null : at - currentFeedStart,
    };
    if (event.type === 'voice.vad')
      trace.push({
        type: event.type,
        state: event.payload.state,
        ...timing,
      });
    if (event.type === 'voice.bargeIn') trace.push({ type: event.type, ...timing });
    if (event.type === 'stage.error') {
      diagnosticCodes.push(event.payload.code);
      if (isFatalVoiceDiagnostic(event.payload.code)) pipelineFailure = event.payload.code;
    }
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
    diagnosticCodes = [];
    pipelineFailure = undefined;
    collecting = true;
    let seq = 0;
    const active = runtime;
    const feedDurations: number[] = [];
    const replay = await replayPcm(
      pcm,
      (block) => {
        currentFeedStart = performance.now();
        try {
          mic.send({ v: 2, seq: seq++, pcm: block });
        } finally {
          feedDurations.push(performance.now() - currentFeedStart);
          currentFeedStart = undefined;
        }
      },
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
      diagnosticCodes: [...diagnosticCodes],
      vadEndReceiptMs: lastEnd
        ? lastEnd.at + lastEnd.stampToCallbackMs - replay.startedAt - ref.speechEndMs
        : null,
      bargeInReceiptMs: firstBarge
        ? firstBarge.at + firstBarge.stampToCallbackMs - replay.startedAt - ref.speechStartMs
        : null,
      processing: {
        feedCallWallMs: sampleStats(feedDurations),
        eventStampToLocalCallbackMs: sampleStats(trace.map((e) => e.stampToCallbackMs)),
        feedStartToEventStampMs: sampleStats(
          trace.flatMap((e) => (e.feedToStampMs === null ? [] : [e.feedToStampMs])),
        ),
        isolatedNativeVadMs: null,
        externalIpcSendMs: null,
      },
      trace: trace.map((e) => ({ ...e, at: e.at - replay.startedAt })),
    });
    console.info(`Replayed ${results.length}/60 ${ref.id}`);
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
  const acceptance = assessLatency(results);
  const { passed } = acceptance;
  const report = {
    measuredAt: new Date().toISOString(),
    status: passed ? 'PASS' : 'FAIL',
    method: bundle.method,
    automaticTimestampGridMs: 80,
    humanSampleResolutionMs: 0.0625,
    referenceUncertainty: bundle.referenceUncertainty,
    automaticReferenceCount: bundle.automaticCount,
    humanReferenceCount: bundle.humanCount,
    referenceSha256: bundle.referenceSha256,
    measurementEndpoint:
      'event.ts inside publish before schema.parse/emit; local callback receipt also reported; physical microphone and external IPC excluded',
    timingInstrumentation:
      'feed wall spans and event stamp to local callback; includes probe overhead, not isolated native inference',
    vadEndDistributionMs: sampleStats(validEnds),
    bargeInDistributionMs: sampleStats(
      results.flatMap((r) => (r.bargeInMs === null ? [] : [r.bargeInMs])),
    ),
    ...acceptance,
    nearThresholdIds: results
      .filter(
        (r) =>
          (r.vadEndMs !== null && Math.abs(r.vadEndMs - 350) <= 80) ||
          (r.bargeInMs !== null && Math.abs(r.bargeInMs - 300) <= 80),
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
