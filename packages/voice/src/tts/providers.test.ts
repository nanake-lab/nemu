import { describe, expect, it, vi } from 'vitest';
import type { TtsRequest } from '@nemu/contracts';
import { LocalTtsProvider } from './local.js';
import { CloudTtsProvider } from './cloud.js';

const request: TtsRequest = {
  text: '你好。',
  lang: 'zh',
  style: 'calm',
  voiceId: 'synthetic',
  sampleRate: 24000,
};

class Socket extends EventTarget {
  readyState = 0;
  binaryType = '';
  sent: string[] = [];
  send(data: string): void {
    this.sent.push(data);
  }
  open(): void {
    this.readyState = 1;
    this.dispatchEvent(new Event('open'));
  }
  pcm(...samples: number[]): void {
    this.dispatchEvent(new MessageEvent('message', { data: Float32Array.from(samples).buffer }));
  }
  close(code = 1000): void {
    this.readyState = 3;
    this.dispatchEvent(Object.assign(new Event('close'), { code }));
  }
}

describe('Local TTS provider', () => {
  it('validates requests, forwards style/language, and decodes PCM', async () => {
    const socket = new Socket();
    const provider = new LocalTtsProvider(undefined, () => socket as unknown as WebSocket);
    const iterable = provider.synthesize(request, new AbortController().signal);
    const iterator = iterable[Symbol.asyncIterator]();
    const first = iterator.next();
    socket.open();
    socket.pcm(0.25, -0.25);
    expect((await first).value).toEqual(Float32Array.from([0.25, -0.25]));
    expect(JSON.parse(socket.sent[0] ?? '')).toEqual(request);
    socket.close();
    expect((await iterator.next()).done).toBe(true);
  });

  it('abort wakes a pending consumer and suppresses queued and future PCM', async () => {
    const socket = new Socket();
    const provider = new LocalTtsProvider(undefined, () => socket as unknown as WebSocket);
    const controller = new AbortController();
    const iterator = provider.synthesize(request, controller.signal)[Symbol.asyncIterator]();
    const pending = iterator.next();
    socket.open();
    controller.abort();
    socket.pcm(0.5);
    expect((await pending).done).toBe(true);
    expect(socket.sent).toContain('{"cancel":true}');
  });

  it('rejects invalid PCM and abnormal close', async () => {
    for (const failure of ['nonfinite', 'close']) {
      const socket = new Socket();
      const iterable = new LocalTtsProvider(
        undefined,
        () => socket as unknown as WebSocket,
      ).synthesize(request, new AbortController().signal);
      const iterator = iterable[Symbol.asyncIterator]();
      const pending = iterator.next();
      socket.open();
      if (failure === 'nonfinite') socket.pcm(NaN);
      else socket.close(1011);
      await expect(pending).rejects.toThrow();
    }
  });

  it('warmup reports degradation', async () => {
    const fetcher = vi.fn<typeof fetch>().mockResolvedValue(Response.json({ ready: false }));
    await expect(new LocalTtsProvider(undefined, undefined, fetcher).warmup()).rejects.toThrow(
      'degraded',
    );
  });
});

describe('OpenAI-compatible TTS (mock only)', () => {
  function provider(fetcher: typeof fetch): CloudTtsProvider {
    return new CloudTtsProvider({
      baseUrl: 'https://tts.invalid/v1/',
      model: 'mock-tts',
      apiKey: async () => 'mock-test-token',
      supportsInstructions: true,
      voices: {
        synthetic: {
          providerVoice: 'synthetic-cloud',
          instructions: { zh: { calm: '平静地说话' } },
        },
      },
      fetcher,
    });
  }

  it('maps voice.json instructions and decodes int16 split across network chunks', async () => {
    const stream = new ReadableStream<Uint8Array>({
      start(controller) {
        controller.enqueue(Uint8Array.from([0]));
        controller.enqueue(Uint8Array.from([128, 255, 127]));
        controller.close();
      },
    });
    const fetcher = vi.fn<typeof fetch>().mockResolvedValue(new Response(stream));
    const pcm: number[] = [];
    for await (const chunk of provider(fetcher).synthesize(request, new AbortController().signal))
      pcm.push(...chunk);
    expect(pcm).toEqual([-1, 32767 / 32768]);
    const [url, options] = fetcher.mock.calls[0] ?? [];
    expect(String(url)).toBe('https://tts.invalid/v1/audio/speech');
    expect(JSON.parse(String(options?.body))).toMatchObject({
      instructions: '平静地说话',
      response_format: 'pcm',
    });
  });

  it('cancels a stalled response on abort', async () => {
    const cancel = vi.fn();
    const stream = new ReadableStream<Uint8Array>({ cancel });
    const controller = new AbortController();
    const iterable = provider(
      vi.fn<typeof fetch>().mockResolvedValue(new Response(stream)),
    ).synthesize(request, controller.signal);
    const iterator = iterable[Symbol.asyncIterator]();
    const pending = iterator.next();
    await vi.waitFor(() => expect(stream.locked).toBe(true));
    controller.abort();
    expect((await pending).done).toBe(true);
    expect(cancel).toHaveBeenCalledOnce();
  });

  it('does not contact the cloud for an unregistered style and rejects truncated audio', async () => {
    const fetcher = vi.fn<typeof fetch>().mockResolvedValue(new Response(Uint8Array.of(1)));
    const client = provider(fetcher);
    const invalid = client.synthesize(
      { ...request, style: 'missing' },
      new AbortController().signal,
    );
    await expect(invalid[Symbol.asyncIterator]().next()).rejects.toThrow('not registered');
    expect(fetcher).not.toHaveBeenCalled();
    const truncated = client.synthesize(request, new AbortController().signal);
    await expect(truncated[Symbol.asyncIterator]().next()).rejects.toThrow('Truncated');
  });
});
