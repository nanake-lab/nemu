import type { Lang } from '@nemu/contracts';
import type { VowelKey } from './types.js';

export interface CrossCorrelationResult {
  bestLagSamples: number;
  delayMs: number;
  peakCorrelation: number;
  correlationCurve: { lagMs: number; value: number }[];
}

/**
 * Computes the normalized cross-correlation between two signals and finds the lag that maximizes correlation.
 *
 * @param audioEnvelope Reference signal (e.g. audio envelope)
 * @param mouthAperture Response signal (e.g. mouth opening curve)
 * @param sampleIntervalMs Time interval between adjacent samples in ms (e.g. 10 ms for 100 Hz sampling)
 * @param maxLagMs Maximum search window for lag in ms (default 200 ms)
 */
export function computeCrossCorrelationLag(
  audioEnvelope: number[],
  mouthAperture: number[],
  sampleIntervalMs = 10,
  maxLagMs = 200,
): CrossCorrelationResult {
  const n = Math.min(audioEnvelope.length, mouthAperture.length);
  if (n < 5) {
    return { bestLagSamples: 0, delayMs: 0, peakCorrelation: 0, correlationCurve: [] };
  }

  // Calculate means and standard deviations
  let sumA = 0;
  let sumB = 0;
  for (let i = 0; i < n; i++) {
    sumA += audioEnvelope[i] ?? 0;
    sumB += mouthAperture[i] ?? 0;
  }
  const meanA = sumA / n;
  const meanB = sumB / n;

  let varA = 0;
  let varB = 0;
  for (let i = 0; i < n; i++) {
    const da = (audioEnvelope[i] ?? 0) - meanA;
    const db = (mouthAperture[i] ?? 0) - meanB;
    varA += da * da;
    varB += db * db;
  }
  const denom = Math.sqrt(varA * varB);
  if (denom < 1e-9) {
    return { bestLagSamples: 0, delayMs: 0, peakCorrelation: 0, correlationCurve: [] };
  }

  const maxLagSamples = Math.floor(maxLagMs / sampleIntervalMs);
  const minLagSamples = -Math.floor(maxLagMs / 4 / sampleIntervalMs); // allow slight negative lag search

  let peakCorr = -Infinity;
  let bestLag = 0;
  const curve: { lagMs: number; value: number }[] = [];

  for (let lag = minLagSamples; lag <= maxLagSamples; lag++) {
    let cross = 0;
    let count = 0;

    for (let i = 0; i < n; i++) {
      const j = i + lag;
      if (j >= 0 && j < n) {
        cross += ((audioEnvelope[i] ?? 0) - meanA) * ((mouthAperture[j] ?? 0) - meanB);
        count++;
      }
    }

    const normCorr = count > 0 ? (cross / denom) * (n / count) : 0;
    const lagMs = lag * sampleIntervalMs;
    curve.push({ lagMs, value: normCorr });

    if (normCorr > peakCorr) {
      peakCorr = normCorr;
      bestLag = lag;
    }
  }

  return {
    bestLagSamples: bestLag,
    delayMs: bestLag * sampleIntervalMs,
    peakCorrelation: peakCorr,
    correlationCurve: curve,
  };
}

export interface SyntheticSentence {
  sentenceId: string;
  text: string;
  lang: Lang;
  sampleRate: number;
  pcm: Float32Array;
  durationSec: number;
}

const FORMANT_FREQS: Record<VowelKey, [number, number, number]> = {
  A: [800, 1200, 2500],
  I: [300, 2300, 3000],
  U: [350, 800, 2200],
  E: [500, 1800, 2600],
  O: [500, 900, 2400],
};

/**
 * Generates synthetic speech audio for a sentence with natural vowel formants, consonant noise, and pauses.
 */
export function generateSyntheticSentence(
  sentenceId: string,
  text: string,
  lang: Lang,
  sampleRate = 16000,
  options?: { vowelsPerSec?: number | undefined; seed?: number | undefined } | undefined,
): SyntheticSentence {
  const vowelsPerSec = options?.vowelsPerSec ?? 4.0;
  const syllableDuration = 1 / vowelsPerSec; // ~250ms per syllable
  const numSyllables = Math.max(3, text.length);
  const totalDurationSec = numSyllables * syllableDuration;
  const totalSamples = Math.round(totalDurationSec * sampleRate);
  const pcm = new Float32Array(totalSamples);

  const vowelSeq: VowelKey[] = ['A', 'I', 'U', 'E', 'O'];
  const baseF0 = lang === 'ja' ? 220 : lang === 'zh' ? 210 : 200; // Pitch

  for (let s = 0; s < numSyllables; s++) {
    const vowel = vowelSeq[s % vowelSeq.length]!;
    const formants = FORMANT_FREQS[vowel];
    const startSample = Math.round(s * syllableDuration * sampleRate);
    const endSample = Math.min(totalSamples, Math.round((s + 1) * syllableDuration * sampleRate));
    const syllableSamples = endSample - startSample;

    // Small consonant onset (first 15% of syllable is noise/transient)
    const consonantLen = Math.round(syllableSamples * 0.15);
    for (let i = 0; i < consonantLen; i++) {
      const noise = (Math.sin(i * 123.456) * 43758.5453) % 1;
      const env = Math.sin((Math.PI * i) / consonantLen) * 0.2;
      pcm[startSample + i] = (noise * 2 - 1) * env;
    }

    // Vowel body (with 3 formants and syllable envelope)
    for (let i = consonantLen; i < syllableSamples; i++) {
      const idx = startSample + i;
      const localT = (i - consonantLen) / (syllableSamples - consonantLen);

      // Syllable attack & decay envelope
      const env = Math.sin(Math.PI * localT);

      // Harmonic excitation
      const t = (i - consonantLen) / sampleRate;
      let sample = 0;

      // Add fundamental + harmonics filtered around formants
      for (let h = 1; h <= 15; h++) {
        const harmonicFreq = baseF0 * h;
        // Formant resonance weighting
        let formantGain = 0;
        for (const f of formants) {
          const bw = f * 0.15;
          const dist = Math.abs(harmonicFreq - f);
          formantGain += 1 / (1 + Math.pow(dist / bw, 2));
        }
        sample += Math.sin(2 * Math.PI * harmonicFreq * t) * (1 / h) * formantGain;
      }

      pcm[idx] = sample * env * 0.25;
    }
  }

  return {
    sentenceId,
    text,
    lang,
    sampleRate,
    pcm,
    durationSec: totalDurationSec,
  };
}
