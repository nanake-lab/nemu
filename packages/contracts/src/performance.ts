import { z } from 'zod';
import { performableMotionIdSchema } from './motions.js';
import { emotionSchema, langSchema, type Lang } from './vocabulary.js';
export { emotionSchema, langSchema } from './vocabulary.js';
export type { Emotion, Lang } from './vocabulary.js';

export const performanceCueSchema = z
  .strictObject({
    kind: z.enum(['emo', 'act', 'look', 'pause', 'voice']),
    value: z.string(),
    intensity: z.number().min(0).max(1).optional(),
  })
  .superRefine((cue, ctx) => {
    const valid =
      cue.kind === 'emo'
        ? emotionSchema.safeParse(cue.value).success
        : cue.kind === 'act'
          ? performableMotionIdSchema.safeParse(cue.value).success
          : cue.kind === 'look'
            ? ['user', 'cursor', 'away', 'screen', 'window'].includes(cue.value)
            : cue.kind === 'voice'
              ? ['whisper', 'excited', 'calm', 'sleepy'].includes(cue.value)
              : /^\d+(?:\.\d+)?$/.test(cue.value) && Number(cue.value) <= 2000;
    if (!valid || (cue.kind !== 'emo' && cue.intensity !== undefined)) {
      ctx.addIssue({ code: 'custom', message: 'Invalid performance cue' });
    }
  });
export type PerformanceCue = z.infer<typeof performanceCueSchema>;
export const performanceSentenceSchema = z.strictObject({
  id: z.string(),
  outputId: z.string(),
  index: z.number().int().nonnegative(),
  lang: langSchema,
  text: z.string(),
  cues: z.array(performanceCueSchema).max(3),
  ttsStyle: z.string(),
});
export type PerformanceSentence = z.infer<typeof performanceSentenceSchema>;

// Case-insensitive and deliberately explicit: adding abbreviations is testable.
export const englishAbbreviations = [
  'mr.',
  'mrs.',
  'ms.',
  'dr.',
  'prof.',
  'sr.',
  'jr.',
  'st.',
  'vs.',
  'e.g.',
  'i.e.',
  'etc.',
  'a.m.',
  'p.m.',
  'u.s.',
  'u.k.',
  'no.',
  'fig.',
] as const;

type Token = { text: string } | { cue: PerformanceCue };
export interface PerformanceParserOptions {
  outputId: string;
  /** Bind to @nemu/log at the application boundary. Never receives raw text. */
  warn: (message: string) => void;
}

function parseCue(raw: string, warn: PerformanceParserOptions['warn']): PerformanceCue | undefined {
  const parts = raw.slice(2, -2).split(':');
  const [kind, value, intensity] = parts;
  const parsed = performanceCueSchema.safeParse({
    kind,
    value,
    ...(intensity !== undefined ? { intensity: intensity.trim() ? Number(intensity) : NaN } : {}),
  });
  if (parts.length < 2 || parts.length > 3 || !parsed.success) {
    warn('Invalid or unknown performance marker ignored');
    return;
  }
  return parsed.data.kind === 'emo'
    ? { ...parsed.data, intensity: parsed.data.intensity ?? 0.6 }
    : parsed.data;
}

async function* characters(
  chunks: AsyncIterable<string> | Iterable<string>,
): AsyncGenerator<string> {
  let highSurrogate = '';
  for await (const chunk of chunks) {
    let text = highSurrogate + chunk;
    highSurrogate = '';
    if (/[\uD800-\uDBFF]$/.test(text)) {
      highSurrogate = text.slice(-1);
      text = text.slice(0, -1);
    }
    yield* text;
  }
  if (highSurrogate) yield highSurrogate;
}

async function* tokenize(
  chunks: AsyncIterable<string> | Iterable<string>,
  warn: PerformanceParserOptions['warn'],
): AsyncGenerator<Token> {
  let marker = '';
  for await (const char of characters(chunks)) {
    if (!marker) {
      if (char === '[') marker = char;
      else yield { text: char };
    } else if (marker === '[' && char !== '[') {
      yield { text: '[' };
      yield { text: char };
      marker = '';
    } else {
      marker += char;
      if (marker.endsWith(']]')) {
        const cue = parseCue(marker, warn);
        if (cue) yield { cue };
        marker = '';
      } else if ([...marker].length > 48) {
        // CONTRACTS 3.2.5 explicitly makes overlong open fragments ordinary text.
        for (const text of marker) yield { text };
        marker = '';
      }
    }
  }
  if (marker === '[') yield { text: marker };
  else if (marker) warn('Unclosed performance marker ignored at end of stream');
}

function language(text: string): Lang {
  const han = (text.match(/\p{Script=Han}/gu) ?? []).length;
  const kana = (text.match(/[\p{Script=Hiragana}\p{Script=Katakana}]/gu) ?? []).length;
  const latin = (text.match(/\p{Script=Latin}/gu) ?? []).length;
  return han + kana > latin ? (kana ? 'ja' : 'zh') : 'en';
}

function abbreviation(text: string): boolean {
  const word = text.match(/([A-Za-z.]+)\.$/)?.[0]?.toLowerCase();
  return word !== undefined && englishAbbreviations.some((item) => word === item);
}

/** A parser is scoped to one output. It consumes lazily and owns no persisted state. */
export function createPerformanceParser(options: PerformanceParserOptions) {
  return async function* parse(
    chunks: AsyncIterable<string> | Iterable<string>,
  ): AsyncGenerator<PerformanceSentence> {
    let text = '';
    let cues: Array<{ offset: number; cue: PerformanceCue }> = [];
    let index = 0;
    let pending: 'cjk' | 'english' | undefined;

    function emit(end = text.length): PerformanceSentence | undefined {
      const sentence = text.slice(0, end).trim();
      const selected = cues
        .filter((item) => item.offset < end)
        .slice(0, 3)
        .map((item) => item.cue);
      cues = cues
        .filter((item) => item.offset >= end)
        .map((item) => ({ ...item, offset: item.offset - end }));
      text = text.slice(end);
      pending = undefined;
      if (!sentence || !/[\p{L}\p{N}\p{S}]/u.test(sentence)) return;
      const style =
        selected.findLast((cue) => cue.kind === 'voice')?.value ??
        selected.findLast((cue) => cue.kind === 'emo')?.value ??
        'neutral';
      return {
        id: `${options.outputId}:${index}`,
        outputId: options.outputId,
        index: index++,
        lang: language(sentence),
        text: sentence,
        cues: selected,
        ttsStyle: style,
      };
    }

    for await (const token of tokenize(chunks, options.warn)) {
      if ('cue' in token) {
        if (pending) {
          const sentence = emit();
          if (sentence) yield sentence;
        }
        cues.push({ offset: text.length, cue: token.cue });
        continue;
      }
      const char = token.text;
      if (pending) {
        if (!/[。！？!?…～~.]/u.test(char)) {
          if (pending === 'cjk' || /\s/u.test(char)) {
            const sentence = emit();
            if (sentence) yield sentence;
          }
          pending = undefined;
        }
      }
      if (!text && /\s/u.test(char)) continue;
      text += char;
      if (char === '\n' || char === '\r') {
        const sentence = emit();
        if (sentence) yield sentence;
        continue;
      }
      const lang = language(text);
      // A mixed punctuation run keeps the rule selected by its first mark.
      if (!pending) {
        if (/[。！？～~]/u.test(char) || (/[!?…]/u.test(char) && lang !== 'en')) pending = 'cjk';
        else if (/[.!?…]/u.test(char) && !abbreviation(text)) pending = 'english';
      }

      // Fast first chunk: 20 CJK characters or 8 complete English words.
      // Later chunks prefer the latest clause separator after 40/25. Without
      // one, enforce the limit with a code-point/whole-word cut.
      const length = [...text.trim()].length;
      const words = text.trim().split(/\s+/u).length;
      let cut = 0;
      if (
        index === 0 &&
        !pending &&
        (lang !== 'en' ? length >= 20 : words >= 8 && /\s/u.test(char))
      )
        cut = text.length;
      else if (index > 0 && (lang !== 'en' ? length > 40 : words > 25)) {
        const separators = [...text.matchAll(lang !== 'en' ? /[，、,；;]/gu : /[,;—]/gu)];
        const last = separators.at(-1);
        if (last) cut = last.index + last[0].length;
        else if (lang !== 'en') cut = [...text].slice(0, 40).join('').length;
        else {
          const wordStarts = [...text.matchAll(/\S+/gu)];
          cut = wordStarts[25]?.index ?? 0;
        }
      }
      if (cut) {
        const sentence = emit(cut);
        if (sentence) yield sentence;
      }
    }
    const final = emit();
    if (final) yield final;
  };
}
