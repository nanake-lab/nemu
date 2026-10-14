import { describe, expect, test, vi } from 'vitest';
import {
  createPerformanceParser,
  englishAbbreviations,
  type PerformanceSentence,
} from './performance.js';

async function collect<T>(source: AsyncIterable<T>): Promise<T[]> {
  const result: T[] = [];
  for await (const item of source) result.push(item);
  return result;
}

async function parse(input: string | string[]) {
  const warn = vi.fn();
  const sentences = await collect(
    createPerformanceParser({ outputId: 'output-1', warn })(
      typeof input === 'string' ? [input] : input,
    ),
  );
  return { sentences, warn, texts: sentences.map((sentence) => sentence.text) };
}

describe('sentence punctuation and language', () => {
  test.each([
    ['中文。下一句。', ['中文。', '下一句。'], 'zh'],
    ['你好！再见！', ['你好！', '再见！'], 'zh'],
    ['你好？再见？', ['你好？', '再见？'], 'zh'],
    ['你好!再见!', ['你好!', '再见!'], 'zh'],
    ['你好?再见?', ['你好?', '再见?'], 'zh'],
    ['你好…再见…', ['你好…', '再见…'], 'zh'],
    ['你好～再见～', ['你好～', '再见～'], 'zh'],
    ['你好~再见~', ['你好~', '再见~'], 'zh'],
    ['你好\n再见', ['你好', '再见'], 'zh'],
    ['こんにちは。お元気？', ['こんにちは。', 'お元気？'], 'ja'],
    ['こんにちは！またね！', ['こんにちは！', 'またね！'], 'ja'],
    ['Hello. Goodbye.', ['Hello.', 'Goodbye.'], 'en'],
    ['Hello! Goodbye!', ['Hello!', 'Goodbye!'], 'en'],
    ['Hello? Goodbye?', ['Hello?', 'Goodbye?'], 'en'],
    ['Hello… Goodbye…', ['Hello…', 'Goodbye…'], 'en'],
    ['Hello... Goodbye...', ['Hello...', 'Goodbye...'], 'en'],
    ['Hello\nGoodbye', ['Hello', 'Goodbye'], 'en'],
    ['Hello\r\nGoodbye', ['Hello', 'Goodbye'], 'en'],
    ['Hello!\tGoodbye', ['Hello!', 'Goodbye'], 'en'],
    ['Hello!\u3000Goodbye', ['Hello!', 'Goodbye'], 'en'],
    ['中文！？…下一句', ['中文！？…', '下一句'], 'zh'],
    ['你好。...Next.', ['你好。...', 'Next.'], 'zh'],
    ['Hello.。Next', ['Hello.。Next'], 'en'],
    ['Hello?!... Goodbye', ['Hello?!...', 'Goodbye'], 'en'],
    ['中文 hello。Next.', ['中文 hello。', 'Next.'], 'en'],
    ['你好世界. Next.', ['你好世界.', 'Next.'], 'zh'],
    ['今日は猫だよ!Next.', ['今日は猫だよ!', 'Next.'], 'ja'],
    ['', [], undefined],
    [' \t\r\n', [], undefined],
    ['。！？\n...', [], undefined],
    ['你好。\n\n再见。', ['你好。', '再见。'], 'zh'],
    ['Hi.There', ['Hi.There'], 'en'],
    ['价格3.14元。', ['价格3.14元。'], 'zh'],
    ['Value 3.14 is exact.', ['Value 3.14 is exact.'], 'en'],
    ['v1.2.3 works.', ['v1.2.3 works.'], 'en'],
    ['  No terminator  ', ['No terminator'], 'en'],
  ] as const)('%s', async (input, expected, lang) => {
    const result = await parse(input);
    expect(result.texts).toEqual(expected);
    expect(result.sentences[0]?.lang).toBe(lang);
  });
  test.each(englishAbbreviations)('does not split abbreviation %s', async (abbr) => {
    const text = `${abbr} Smith is here.`;
    expect((await parse(text)).texts).toEqual([text]);
  });
  test.each(['Mr.', 'Mrs.', 'Dr.', 'E.G.', 'I.E.', 'U.S.'])(
    'case-insensitive abbreviation %s',
    async (abbr) => {
      const text = `${abbr} Smith is here.`;
      expect((await parse([...text])).texts).toEqual([text]);
    },
  );
});

describe('performance markers', () => {
  test.each([
    ['emo:happy', { kind: 'emo', value: 'happy', intensity: 0.6 }, 'happy'],
    ['emo:sad:0', { kind: 'emo', value: 'sad', intensity: 0 }, 'sad'],
    ['emo:excited:1', { kind: 'emo', value: 'excited', intensity: 1 }, 'excited'],
    ['act:wave', { kind: 'act', value: 'wave' }, 'neutral'],
    ['act:react_pout', { kind: 'act', value: 'react_pout' }, 'neutral'],
    ['look:user', { kind: 'look', value: 'user' }, 'neutral'],
    ['look:screen', { kind: 'look', value: 'screen' }, 'neutral'],
    ['look:window', { kind: 'look', value: 'window' }, 'neutral'],
    ['look:cursor', { kind: 'look', value: 'cursor' }, 'neutral'],
    ['look:away', { kind: 'look', value: 'away' }, 'neutral'],
    ['pause:0', { kind: 'pause', value: '0' }, 'neutral'],
    ['pause:2000', { kind: 'pause', value: '2000' }, 'neutral'],
    ['voice:whisper', { kind: 'voice', value: 'whisper' }, 'whisper'],
    ['voice:excited', { kind: 'voice', value: 'excited' }, 'excited'],
    ['voice:calm', { kind: 'voice', value: 'calm' }, 'calm'],
    ['voice:sleepy', { kind: 'voice', value: 'sleepy' }, 'sleepy'],
  ])('strips and attaches %s', async (marker, cue, style) => {
    const { sentences, warn } = await parse(`[[${marker}]]你好。`);
    expect(sentences).toHaveLength(1);
    expect(sentences[0]).toMatchObject({ text: '你好。', cues: [cue], ttsStyle: style });
    expect(warn).not.toHaveBeenCalled();
  });
  test.each([
    'unknown:value',
    'emo:nope',
    'emo:happy:2',
    'emo:happy:-1',
    'emo:happy:NaN',
    'emo:happy:',
    'emo:happy:0.5:extra',
    'act:idle_stand',
    'act:fidget_yawn',
    'act:wave:1',
    'look:nope',
    'pause:-1',
    'pause:2001',
    'pause:no',
    'voice:loud',
    'missing',
  ])('ignores invalid %s with a redacted warning', async (marker) => {
    const { texts, sentences, warn } = await parse(`[[${marker}]]秘密。`);
    expect(texts).toEqual(['秘密。']);
    expect(sentences[0]?.cues).toEqual([]);
    expect(warn).toHaveBeenCalledExactlyOnceWith('Invalid or unknown performance marker ignored');
  });
  test('keeps the first three valid cues and ignores the rest', async () => {
    const { sentences } = await parse(
      '[[look:user]][[act:wave]][[emo:happy]][[voice:whisper]]你好。',
    );
    expect(sentences[0]?.cues).toHaveLength(3);
    expect(sentences[0]?.ttsStyle).toBe('happy');
  });
  test('invalid markers do not consume the valid cue budget', async () => {
    const { sentences } = await parse('[[bad:a]][[look:user]][[act:wave]][[emo:happy]]你好。');
    expect(sentences[0]?.cues).toHaveLength(3);
  });
  test('voice takes precedence over emotion regardless of order', async () => {
    expect((await parse('[[voice:calm]][[emo:happy]]你好。')).sentences[0]?.ttsStyle).toBe('calm');
  });
  test('last accepted cue of the same style kind wins', async () => {
    expect((await parse('[[voice:calm]][[voice:whisper]]你好。')).sentences[0]?.ttsStyle).toBe(
      'whisper',
    );
  });
  test.each(['你好。', 'Hello.', 'Hello!', 'Hello?', 'Hello...', 'こんにちは！'])(
    'marker after punctuation belongs to next sentence: %s',
    async (first) => {
      const { sentences } = await parse(`${first}[[act:wave]]接着说。`);
      expect(sentences.map((s) => s.cues)).toEqual([[], [{ kind: 'act', value: 'wave' }]]);
    },
  );
  test('marker within a sentence remains on that sentence', async () => {
    expect((await parse('你[[act:nod]]好。')).sentences[0]).toMatchObject({
      text: '你好。',
      cues: [{ kind: 'act', value: 'nod' }],
    });
  });
  test('cue-only stream has no empty sentence', async () => {
    expect((await parse('[[act:wave]]')).sentences).toEqual([]);
  });
  test('unfinished marker at EOF is not spoken', async () => {
    const result = await parse('你好[[emo:happy');
    expect(result.texts).toEqual(['你好']);
    expect(result.warn).toHaveBeenCalledOnce();
  });
  test('a lone opening bracket is ordinary text', async () => {
    expect((await parse('Text [')).texts).toEqual(['Text [']);
  });
  test('ordinary bracket expressions are preserved', async () => {
    expect((await parse('Hello [world].')).texts).toEqual(['Hello [world].']);
  });
  test('48-character unfinished marker remains buffered', async () => {
    const result = await parse('[[' + 'x'.repeat(46));
    expect(result.texts).toEqual([]);
    expect(result.warn).toHaveBeenCalledOnce();
  });
  test('49-character unfinished marker becomes ordinary text', async () => {
    const input = '[[' + 'x'.repeat(47);
    expect((await parse([...input])).texts).toEqual([input]);
  });
  test('overlong fragment recovers and recognizes subsequent markers', async () => {
    const input = '[[' + 'x'.repeat(47) + '[[act:wave]] done.';
    expect((await parse(input)).sentences[0]).toMatchObject({
      text: '[[' + 'x'.repeat(47) + ' done.',
      cues: [{ kind: 'act', value: 'wave' }],
    });
  });
});

describe('streaming and length limits', () => {
  const streamed = '[[emo:happy:0.8]]你好。[[voice:calm]]Dr. Smith says hello... Next!';
  test.each(Array.from({ length: streamed.length + 1 }, (_, index) => index))(
    'same result with boundary at %i',
    async (offset) => {
      const expected = await parse(streamed);
      expect((await parse([streamed.slice(0, offset), streamed.slice(offset)])).sentences).toEqual(
        expected.sentences,
      );
    },
  );
  test('single-character chunks preserve the same result', async () => {
    expect((await parse([...streamed])).sentences).toEqual((await parse(streamed)).sentences);
  });
  test('UTF-16 surrogate pair split between chunks is preserved', async () => {
    expect((await parse(['你\ud83d', '\ude00好。'])).texts).toEqual(['你😀好。']);
  });
  test('a trailing unpaired surrogate is preserved', async () => {
    expect((await parse(['Hello\ud83d'])).texts).toEqual(['Hello\ud83d']);
  });
  test('empty chunks are harmless', async () => {
    expect((await parse(['', '你', '', '好。', ''])).texts).toEqual(['你好。']);
  });
  test('Chinese first sentence is emitted at 20 characters without waiting for EOF', async () => {
    let reachedEnd = false;
    async function* source() {
      yield '你'.repeat(20);
      reachedEnd = true;
      yield '好。';
    }
    const iterator = createPerformanceParser({ outputId: 'o', warn: vi.fn() })(source());
    expect((await iterator.next()).value?.text).toBe('你'.repeat(20));
    expect(reachedEnd).toBe(false);
    await iterator.return(undefined);
  });
  test('short Chinese sentence is not held to 20 characters', async () => {
    expect((await parse('你好。再见。')).texts).toEqual(['你好。', '再见。']);
  });
  test('English first sentence emits at eight complete words', async () => {
    const input = 'one two three four five six seven eight nine ten.';
    expect((await parse(input)).texts).toEqual([
      'one two three four five six seven eight',
      'nine ten.',
    ]);
  });
  test('short English sentence is not held to eight words', async () => {
    expect((await parse('Hi. Next.')).texts).toEqual(['Hi.', 'Next.']);
  });
  test.each(['，', '、', ',', '；', ';'])(
    'Chinese over 40 characters splits at latest %s',
    async (sep) => {
      const left = '中'.repeat(25) + sep;
      const right = '文'.repeat(20) + '。';
      expect((await parse('先。' + left + right)).texts).toEqual(['先。', left, right]);
    },
  );
  test.each([',', ';', '—'])('English over 25 words splits at latest %s', async (sep) => {
    const left = Array(15).fill('word').join(' ') + sep;
    const right = Array(12).fill('next').join(' ') + '.';
    expect((await parse('Hi. ' + left + ' ' + right)).texts).toEqual(['Hi.', left, right]);
  });
  test('without a clause separator the Chinese 40-character limit still applies', async () => {
    expect((await parse('先。' + '中'.repeat(50) + '。')).texts).toEqual([
      '先。',
      '中'.repeat(40),
      '中'.repeat(10) + '。',
    ]);
  });
  test('without a clause separator English cuts after 25 complete words', async () => {
    const words = Array(28).fill('word');
    expect((await parse('Hi. ' + words.join(' ') + '.')).texts).toEqual([
      'Hi.',
      words.slice(0, 25).join(' '),
      words.slice(25).join(' ') + '.',
    ]);
  });
  test('cues after a forced clause cut stay with the remainder', async () => {
    const result = await parse(
      '先。' + '中'.repeat(25) + '，[[act:wave]]' + '文'.repeat(20) + '。',
    );
    expect(result.sentences.map((s) => s.cues)).toEqual([[], [], [{ kind: 'act', value: 'wave' }]]);
  });
  test('output identifiers and indices are stable and scoped per parser invocation', async () => {
    const result = await parse('你。好。');
    expect(result.sentences.map(({ id, outputId, index }) => ({ id, outputId, index }))).toEqual([
      { id: 'output-1:0', outputId: 'output-1', index: 0 },
      { id: 'output-1:1', outputId: 'output-1', index: 1 },
    ]);
  });
  test('upstream errors propagate without fabricating completion', async () => {
    async function* source() {
      yield 'Hi';
      throw new Error('upstream failed');
    }
    await expect(
      collect(createPerformanceParser({ outputId: 'o', warn: vi.fn() })(source())),
    ).rejects.toThrow('upstream failed');
  });
  test('consumer cancellation closes the upstream iterator', async () => {
    let closed = false;
    async function* source() {
      try {
        yield '你'.repeat(20);
        yield '更多';
      } finally {
        closed = true;
      }
    }
    const stream = createPerformanceParser({ outputId: 'o', warn: vi.fn() })(source());
    await stream.next();
    await stream.return(undefined);
    expect(closed).toBe(true);
  });
  test('parser outputs validate with the public schema', async () => {
    const { performanceSentenceSchema } = await import('./performance.js');
    const sentences: PerformanceSentence[] = (await parse(streamed)).sentences;
    for (const sentence of sentences)
      expect(performanceSentenceSchema.safeParse(sentence).success).toBe(true);
  });
});
