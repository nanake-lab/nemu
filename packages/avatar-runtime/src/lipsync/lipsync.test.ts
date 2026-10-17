import { describe, it, expect, vi } from 'vitest';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import type { Lang, PerformanceCue } from '@nemu/contracts';
import { FacialLayer } from '../animation/L3-face.js';
import { VisemeSmoother } from './smoother.js';
import { StageAudioQueue } from './audio-queue.js';
import { LipSyncRuntime } from './runtime.js';
import { LipSyncAnalyser } from './analyser.js';
import { DEFAULT_EN_PROFILE, DEFAULT_JA_PROFILE, DEFAULT_ZH_PROFILE } from './profiles.js';
import {
  computeCrossCorrelationLag,
  generateSyntheticSentence,
} from './cross-correlation.js';

describe('VisemeSmoother', () => {
  it('smooths mouth opening with ~40ms attack time constant', () => {
    const smoother = new VisemeSmoother({ attackTimeMs: 40, decayTimeMs: 80, minVolumeDb: -40 });

    // Step 0: at rest
    expect(smoother.getVisemes().aa).toBe(0);

    // Step at 40ms with target A = 1.0 (volume = -10 dB, above threshold)
    const result40ms = smoother.update({ A: 1.0 }, -10, 0.040);
    // Exponential approach: 1 - e^(-40/40) = 1 - e^(-1) ~= 0.632
    expect(result40ms.aa).toBeCloseTo(0.632, 2);

    // After another 40ms (total 80ms)
    const result80ms = smoother.update({ A: 1.0 }, -10, 0.040);
    // 0.632 + (1 - 0.632) * (1 - e^(-1)) ~= 0.865
    expect(result80ms.aa).toBeCloseTo(0.865, 2);
  });

  it('smooths mouth closing with ~80ms decay time constant', () => {
    const smoother = new VisemeSmoother({ attackTimeMs: 40, decayTimeMs: 80, minVolumeDb: -40 });

    // Open mouth fully
    for (let i = 0; i < 20; i++) {
      smoother.update({ A: 1.0 }, -10, 0.050);
    }
    expect(smoother.getVisemes().aa).toBeGreaterThan(0.99);

    // Release to 0 with volume active
    const decay80ms = smoother.update({ A: 0.0 }, -10, 0.080);
    // Exponential decay: e^(-80/80) = e^(-1) ~= 0.368
    expect(decay80ms.aa).toBeCloseTo(0.368, 2);

    // After another 80ms (total 160ms)
    const decay160ms = smoother.update({ A: 0.0 }, -10, 0.080);
    // 0.368 * e^(-1) ~= 0.135
    expect(decay160ms.aa).toBeCloseTo(0.135, 2);
  });

  it('closes mouth when volume drops below silence threshold (-40 dB)', () => {
    const smoother = new VisemeSmoother({ attackTimeMs: 40, decayTimeMs: 80, minVolumeDb: -40 });

    // Open mouth
    for (let i = 0; i < 10; i++) {
      smoother.update({ A: 1.0 }, -15, 0.040);
    }
    expect(smoother.getVisemes().aa).toBeGreaterThan(0.9);

    // Now volume drops to -50 dB (below silence threshold of -40 dB)
    // Even though raw target is A = 1.0, threshold gating forces target to 0
    smoother.update({ A: 1.0 }, -50, 0.080);
    expect(smoother.getVisemes().aa).toBeLessThan(0.4);

    // Decays toward 0
    for (let i = 0; i < 5; i++) {
      smoother.update({ A: 1.0 }, -50, 0.080);
    }
    expect(smoother.getVisemes().aa).toBeLessThan(0.01);
  });

  it('computes mouth aperture properly across different visemes', () => {
    const smoother = new VisemeSmoother();
    smoother.update({ A: 1.0 }, -10, 0.5);
    const apA = smoother.getMouthAperture();
    expect(apA).toBeGreaterThan(0.9);

    smoother.reset();
    smoother.update({ U: 1.0 }, -10, 0.5);
    const apU = smoother.getMouthAperture();
    expect(apU).toBeLessThan(apA); // 'ou' aperture is smaller than 'aa'
  });
});

describe('StageAudioQueue', () => {
  it('queues chunks and emits started, progress, and done events', () => {
    const startedFn = vi.fn();
    const progressFn = vi.fn();
    const doneFn = vi.fn();
    const audioStartMarkFn = vi.fn();

    const queue = new StageAudioQueue({
      onSpeechStarted: startedFn,
      onSpeechProgress: progressFn,
      onSpeechDone: doneFn,
      onAudioStartMark: audioStartMarkFn,
      clock: () => 1000,
    });

    const testCue: PerformanceCue = { kind: 'emo', value: 'happy', intensity: 0.8 };
    queue.handleSpeechBegin({
      outputId: 'out-1',
      chunkIndex: 0,
      sentenceId: 'sent-1',
      text: '你好世界！',
      lang: 'zh',
      cues: [testCue],
      sampleRate: 16000,
    });

    const pcm1 = new Float32Array(8000).fill(0.1); // 0.5 sec
    const pcm2 = new Float32Array(8000).fill(0.2); // 0.5 sec
    queue.handleSpeechChunk({ sentenceId: 'sent-1', pcm: pcm1 });
    queue.handleSpeechChunk({ sentenceId: 'sent-1', pcm: pcm2 });
    queue.handleSpeechEnd({ sentenceId: 'sent-1' });

    // First step triggers started event and audioStart mark
    const step1 = queue.processStep(0.1);
    expect(startedFn).toHaveBeenCalledWith(
      { sentenceId: 'sent-1', at: 1000 },
      [testCue],
    );
    expect(audioStartMarkFn).toHaveBeenCalledWith(1000);
    expect(step1.isSpeaking).toBe(true);
    expect(progressFn).toHaveBeenCalled();

    // Advance remaining 0.9 sec
    queue.processStep(0.9);
    expect(doneFn).toHaveBeenCalledWith({
      sentenceId: 'sent-1',
      interrupted: false,
      playedChars: 5,
    });
  });

  it('handles interruption: volume drops below -40 dB within 50ms and tracks playedChars accurately', async () => {
    const doneFn = vi.fn();
    const queue = new StageAudioQueue({
      onSpeechDone: doneFn,
    });

    // 10 characters text, 2 seconds audio at 16000 Hz = 32000 samples
    const text = '0123456789';
    queue.handleSpeechBegin({
      outputId: 'out-1',
      chunkIndex: 0,
      sentenceId: 'sent-interrupt',
      text,
      lang: 'zh',
      cues: [],
      sampleRate: 16000,
    });

    const pcm = new Float32Array(32000).fill(0.5);
    queue.handleSpeechChunk({ sentenceId: 'sent-interrupt', pcm });
    queue.handleSpeechEnd({ sentenceId: 'sent-interrupt' });

    // Play 1.0 second (50% through the sentence -> playedChars should be ~5)
    queue.processStep(1.0);

    // Now trigger interruption with fadeMs = 40 ms
    queue.handleSpeechStop({ fadeMs: 40 });

    const fadeTrace: Array<{ timeMs: number; gain: number; volumeDb: number }> = [];

    // Track progression every 5 ms
    for (let t = 5; t <= 50; t += 5) {
      const step = queue.processStep(0.005);
      fadeTrace.push({
        timeMs: t,
        gain: Number(step.gain.toFixed(4)),
        volumeDb: Number(step.volumeDb.toFixed(2)),
      });
    }

    // Step at 45ms into fade must already be below -40 dB
    const at45ms = fadeTrace.find((f) => f.timeMs >= 40);
    expect(at45ms?.volumeDb).toBeLessThanOrEqual(-40);

    // Verify done event was emitted with interrupted: true and accurate playedChars
    expect(doneFn).toHaveBeenCalledWith(
      expect.objectContaining({
        sentenceId: 'sent-interrupt',
        interrupted: true,
      }),
    );

    const callArgs = doneFn.mock.calls[0]![0];
    // Exactly around 5 characters (error <= 1 char)
    expect(Math.abs(callArgs.playedChars - 5)).toBeLessThanOrEqual(1);

    // Write interruption benchmark evidence
    const reportDir = resolve(process.cwd(), 'reports/P4-04');
    await mkdir(reportDir, { recursive: true });
    await writeFile(
      resolve(reportDir, 'interruption_benchmark.json'),
      JSON.stringify(
        {
          sentenceId: 'sent-interrupt',
          totalChars: 10,
          playedChars: callArgs.playedChars,
          expectedChars: 5,
          charError: Math.abs(callArgs.playedChars - 5),
          interrupted: true,
          fadeMsRequested: 40,
          fadeTrace,
          volumeAt40msDb: at45ms?.volumeDb,
          volumeThresholdDb: -40,
          passedInterruptionThreshold: (at45ms?.volumeDb ?? 0) <= -40,
        },
        null,
        2,
      ),
      'utf-8',
    );
  });
});

describe('LipSyncAnalyser & Profiles', () => {
  it('extracts MFCC and detects vowel similarities for zh, en, ja profiles', () => {
    const zhAnalyser = new LipSyncAnalyser(DEFAULT_ZH_PROFILE);
    const enAnalyser = new LipSyncAnalyser(DEFAULT_EN_PROFILE);
    const jaAnalyser = new LipSyncAnalyser(DEFAULT_JA_PROFILE);

    expect(zhAnalyser.getProfile()).toBe(DEFAULT_ZH_PROFILE);
    expect(enAnalyser.getProfile()).toBe(DEFAULT_EN_PROFILE);
    expect(jaAnalyser.getProfile()).toBe(DEFAULT_JA_PROFILE);

    // Synthetic A sound
    const synthA = generateSyntheticSentence('test', 'AAAA', 'zh');
    const frame = synthA.pcm.subarray(1000, 1512);

    const res = zhAnalyser.analyzeFrame(frame);
    expect(res.volumeDb).toBeGreaterThan(-40);
    expect(res.mfcc.length).toBe(12);
    // Vowel scores should have active non-zero values
    expect(res.vowelScores.A).toBeGreaterThan(0);
  });
});

describe('LipSyncRuntime integration with L3 Face Layer', () => {
  it('switches language profile dynamically and drives L3 visemes with mouth channel priority', () => {
    const facialLayer = new FacialLayer();
    const runtime = new LipSyncRuntime();
    runtime.bindFacialLayer(facialLayer);

    expect(runtime.getCurrentLanguage()).toBe('zh');

    // Speech begin in English switches profile to en
    runtime.handleSpeechBegin({
      outputId: 'out-en',
      chunkIndex: 0,
      sentenceId: 'sent-en',
      text: 'Hello world',
      lang: 'en',
      cues: [],
      sampleRate: 16000,
    });
    expect(runtime.getCurrentLanguage()).toBe('en');

    // Generate synthetic audio for 'Hello world'
    const synth = generateSyntheticSentence('sent-en', 'Hello world', 'en');
    runtime.handleSpeechChunk({ sentenceId: 'sent-en', pcm: synth.pcm });
    runtime.handleSpeechEnd({ sentenceId: 'sent-en' });

    // Step runtime and verify L3 mouth priority
    let maxAperture = 0;
    for (let t = 0; t < 10; t++) {
      const visemes = runtime.update(0.05); // 50ms step
      const aperture = runtime.getMouthAperture();
      if (aperture > maxAperture) maxAperture = aperture;
      expect(visemes).toBeDefined();
    }

    expect(maxAperture).toBeGreaterThan(0.05);
    // In FacialLayer, speaking state should be active during viseme activation
    expect(facialLayer.getIsSpeaking()).toBe(true);
  });
});

describe('Cross-correlation deviation test (完成标准第 10 项)', () => {
  const languages: Lang[] = ['zh', 'en', 'ja'];

  // Test sentences for each language (20 sentences each)
  const sampleSentences: Record<Lang, string[]> = {
    zh: [
      '你好呀，很高兴见到你。',
      '今天天气真不错呢。',
      '桌面伙伴随时为你服务。',
      '你想要听一首歌吗？',
      '要不要一起看一会儿书？',
      '工作辛苦了，喝口水休息一下吧。',
      '代码写得非常漂亮！',
      '今天有什么新计划吗？',
      '不要熬夜太晚哦。',
      '明天会是充满希望的一天。',
      '风吹过窗台，感觉好舒服。',
      '你今天笑起来真好看。',
      '有什么心事可以跟我聊聊。',
      '这道难题我们一定能解开。',
      '慢慢来，不用着急。',
      '我一直在这里陪着你。',
      '记得按时吃午饭呀。',
      '深呼吸，放松一下肩膀。',
      '星星今晚好明亮。',
      '晚安，愿你有个好梦。',
    ],
    en: [
      'Hello there, so wonderful to meet you.',
      'How has your day been going so far?',
      'Let us make today truly remarkable.',
      'Remember to take a breath and relax.',
      'Great work on solving that problem!',
      'I will be right here whenever you need me.',
      'The morning sun looks so beautiful today.',
      'Keep up the fantastic momentum.',
      'Are you ready for the next challenge?',
      'Stay curious and keep exploring forward.',
      'That was an impressive achievement.',
      'Let us review the plan step by step.',
      'Do not forget to stay hydrated today.',
      'Everything is going according to plan.',
      'Take your time, there is no need to rush.',
      'Music always makes everything feel better.',
      'You are doing great, keep going!',
      'Look at how much progress you have made.',
      'Have a restful evening and sweet dreams.',
      'See you again tomorrow morning!',
    ],
    ja: [
      'こんにちは、お会いできて嬉しいです。',
      '今日も一日、頑張りましょうね。',
      '何か困ったことがあれば呼んでください。',
      '少し休憩して、お茶でもいかがですか？',
      '素晴らしい成果が出ましたね！',
      'あなたの笑顔を見られて良かったです。',
      '無理をせず、マイペースでいきましょう。',
      '今日の空はとても澄み渡っています。',
      'いつでもそばで応援していますよ。',
      '一歩ずつ着実に進んでいますね。',
      '温かいスープを飲んで温まってください。',
      '今日のタスクも順調に進んでいます。',
      '夜風がとても心地いいですね。',
      '深呼吸をしてリラックスしてください。',
      '明日もきっといい日になりますよ。',
      'お疲れ様でした、よく頑張りましたね。',
      '好きな音楽を聴いて気分転換しましょう。',
      '星空がとても綺麗に輝いています。',
      '今夜はゆっくり休んでくださいね。',
      'おやすみなさい、良い夢を。',
    ],
  };

  const allLangResults: Record<
    Lang,
    {
      meanDelayMs: number;
      maxDelayMs: number;
      minDelayMs: number;
      meanCorrelation: number;
      sentences: Array<{
        sentenceId: string;
        text: string;
        delayMs: number;
        peakCorrelation: number;
      }>;
    }
  > = {
    zh: { meanDelayMs: 0, maxDelayMs: 0, minDelayMs: 0, meanCorrelation: 0, sentences: [] },
    en: { meanDelayMs: 0, maxDelayMs: 0, minDelayMs: 0, meanCorrelation: 0, sentences: [] },
    ja: { meanDelayMs: 0, maxDelayMs: 0, minDelayMs: 0, meanCorrelation: 0, sentences: [] },
  };

  for (const lang of languages) {
    it(`cross-correlation delay for 20 sentences in ${lang}: mean <= 50ms, max <= 80ms`, async () => {
      const sentences = sampleSentences[lang];
      expect(sentences.length).toBe(20);

      const delays: number[] = [];
      const correlations: number[] = [];
      const sentenceRecords: Array<{
        sentenceId: string;
        text: string;
        delayMs: number;
        peakCorrelation: number;
      }> = [];

      for (let sIdx = 0; sIdx < sentences.length; sIdx++) {
        const text = sentences[sIdx]!;
        const synth = generateSyntheticSentence(`sent-${lang}-${sIdx}`, text, lang, 16000, {
          vowelsPerSec: 4.5,
        });

        const runtime = new LipSyncRuntime({
          sampleRate: 16000,
          attackTimeMs: 40,
          decayTimeMs: 80,
          minVolumeDb: -40,
        });

        runtime.handleSpeechBegin({
          outputId: `out-${lang}-${sIdx}`,
          chunkIndex: 0,
          sentenceId: synth.sentenceId,
          text,
          lang,
          cues: [],
          sampleRate: 16000,
        });
        runtime.handleSpeechChunk({ sentenceId: synth.sentenceId, pcm: synth.pcm });
        runtime.handleSpeechEnd({ sentenceId: synth.sentenceId });

        // Record audio envelope and mouth aperture in 10ms (100 Hz) time steps
        const stepSec = 0.010; // 10ms
        const totalSteps = Math.floor(synth.durationSec / stepSec);

        const envelope: number[] = [];
        const mouthAperture: number[] = [];

        for (let step = 0; step < totalSteps; step++) {
          runtime.update(stepSec);

          const volDb = runtime.getAudioVolumeDb();
          // Normalized audio envelope in [0..1]
          const env = volDb < -40 ? 0 : Math.min(1.0, Math.max(0, (volDb + 40) / 30));
          const aperture = runtime.getMouthAperture();

          envelope.push(env);
          mouthAperture.push(aperture);
        }

        const xcorr = computeCrossCorrelationLag(envelope, mouthAperture, 10, 200);
        delays.push(xcorr.delayMs);
        correlations.push(xcorr.peakCorrelation);
        sentenceRecords.push({
          sentenceId: synth.sentenceId,
          text,
          delayMs: xcorr.delayMs,
          peakCorrelation: Number(xcorr.peakCorrelation.toFixed(4)),
        });

        // Individual sentence maximum constraint
        expect(xcorr.delayMs).toBeLessThanOrEqual(80);
      }

      const meanDelay = delays.reduce((a, b) => a + b, 0) / delays.length;
      const maxDelay = Math.max(...delays);
      const minDelay = Math.min(...delays);
      const meanCorr = correlations.reduce((a, b) => a + b, 0) / correlations.length;

      allLangResults[lang] = {
        meanDelayMs: Number(meanDelay.toFixed(2)),
        maxDelayMs: maxDelay,
        minDelayMs: minDelay,
        meanCorrelation: Number(meanCorr.toFixed(4)),
        sentences: sentenceRecords,
      };

      // Verify acceptance thresholds (mean <= 50ms, max <= 80ms)
      expect(meanDelay).toBeLessThanOrEqual(50);
      expect(maxDelay).toBeLessThanOrEqual(80);
      expect(meanDelay).toBeGreaterThanOrEqual(10); // Mouth realistically lags audio envelope

      // If finished all 3 languages, write raw benchmark artifact
      if (lang === 'ja') {
        const reportDir = resolve(process.cwd(), 'reports/P4-04');
        await mkdir(reportDir, { recursive: true });
        await writeFile(
          resolve(reportDir, 'xcorr_benchmark.json'),
          JSON.stringify(
            {
              thresholds: {
                maxAllowedMeanDelayMs: 50,
                maxAllowedMaxDelayMs: 80,
              },
              summary: {
                zh: {
                  meanDelayMs: allLangResults.zh.meanDelayMs,
                  maxDelayMs: allLangResults.zh.maxDelayMs,
                  minDelayMs: allLangResults.zh.minDelayMs,
                  meanCorrelation: allLangResults.zh.meanCorrelation,
                  passed: allLangResults.zh.meanDelayMs <= 50 && allLangResults.zh.maxDelayMs <= 80,
                },
                en: {
                  meanDelayMs: allLangResults.en.meanDelayMs,
                  maxDelayMs: allLangResults.en.maxDelayMs,
                  minDelayMs: allLangResults.en.minDelayMs,
                  meanCorrelation: allLangResults.en.meanCorrelation,
                  passed: allLangResults.en.meanDelayMs <= 50 && allLangResults.en.maxDelayMs <= 80,
                },
                ja: {
                  meanDelayMs: allLangResults.ja.meanDelayMs,
                  maxDelayMs: allLangResults.ja.maxDelayMs,
                  minDelayMs: allLangResults.ja.minDelayMs,
                  meanCorrelation: allLangResults.ja.meanCorrelation,
                  passed: allLangResults.ja.meanDelayMs <= 50 && allLangResults.ja.maxDelayMs <= 80,
                },
              },
              detailedSentences: allLangResults,
            },
            null,
            2,
          ),
          'utf-8',
        );
      }
    });
  }
});
