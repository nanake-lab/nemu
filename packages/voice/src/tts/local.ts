import { ttsRequestSchema, type TtsProvider, type TtsRequest } from '@nemu/contracts';

/** One request per socket. Binary frames are little-endian mono float32 at 24 kHz. */
export class LocalTtsProvider implements TtsProvider {
  readonly id = 'qwen3-tts-local';

  constructor(
    private readonly baseUrl = 'http://127.0.0.1:17863',
    private readonly socketFactory = (url: string): WebSocket => new WebSocket(url),
    private readonly fetcher: typeof fetch = fetch,
  ) {}

  async warmup(): Promise<void> {
    const response = await this.fetcher(new URL('/warmup', this.baseUrl), {
      method: 'POST',
      signal: AbortSignal.timeout(120_000),
    });
    const health: unknown = await response.json();
    if (
      !response.ok ||
      typeof health !== 'object' ||
      health === null ||
      !('ready' in health) ||
      health.ready !== true
    ) {
      throw new Error('TTS sidecar is degraded; inspect /health');
    }
  }

  async *synthesize(input: TtsRequest, signal: AbortSignal): AsyncIterable<Float32Array> {
    const request = ttsRequestSchema.parse(input);
    if (signal.aborted) return;
    const url = new URL('/synthesize', this.baseUrl);
    url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
    const ws = this.socketFactory(url.href);
    ws.binaryType = 'arraybuffer';
    const queue: Float32Array[] = [];
    let ended = false;
    let failure: Error | undefined;
    let wake: (() => void) | undefined;
    const notify = (): void => {
      wake?.();
      wake = undefined;
    };
    const finish = (error?: Error): void => {
      ended = true;
      failure = failure ?? error;
      notify();
    };
    const abort = (): void => {
      queue.length = 0;
      if (ws.readyState === WebSocket.OPEN) ws.send(JSON.stringify({ cancel: true }));
      // close CONNECTING may throw in alternative WebSocket implementations.
      try {
        ws.close();
      } catch {
        /* abort still ends the local iterator */
      }
      finish();
    };
    const timer = setTimeout(() => {
      finish(new Error('TTS socket timed out'));
      ws.close();
    }, 120_000);
    const onOpen = (): void => {
      if (signal.aborted) {
        abort();
        return;
      }
      ws.send(JSON.stringify(request));
    };
    const onMessage = (event: MessageEvent<unknown>): void => {
      if (signal.aborted || ended) return;
      if (
        !(event.data instanceof ArrayBuffer) ||
        event.data.byteLength % 4 !== 0 ||
        event.data.byteLength > 1_048_576 ||
        queue.length >= 8
      ) {
        finish(new Error('Invalid or overflowing TTS PCM stream'));
        ws.close();
        return;
      }
      const view = new DataView(event.data);
      const pcm = new Float32Array(view.byteLength / 4);
      for (let i = 0; i < pcm.length; i++) {
        const sample = view.getFloat32(i * 4, true);
        if (!Number.isFinite(sample)) {
          finish(new Error('Non-finite TTS PCM'));
          ws.close();
          return;
        }
        pcm[i] = sample;
      }
      queue.push(pcm);
      notify();
    };
    const onClose = (event: CloseEvent): void => {
      if (!signal.aborted)
        finish(event.code === 1000 ? undefined : new Error(`TTS closed: ${event.code}`));
    };
    const onError = (): void => {
      if (!signal.aborted) finish(new Error('TTS socket failed'));
    };
    ws.addEventListener('open', onOpen);
    ws.addEventListener('message', onMessage);
    ws.addEventListener('close', onClose);
    ws.addEventListener('error', onError);
    signal.addEventListener('abort', abort, { once: true });
    if (signal.aborted) abort();
    try {
      while (!signal.aborted) {
        if (failure) throw failure;
        const pcm = queue.shift();
        if (pcm) {
          yield pcm;
          continue;
        }
        if (ended) break;
        await new Promise<void>((resolve) => {
          wake = resolve;
        });
      }
    } finally {
      clearTimeout(timer);
      signal.removeEventListener('abort', abort);
      if (ws.readyState === WebSocket.OPEN && !ended) ws.send(JSON.stringify({ cancel: true }));
      // Retain error listener until close to absorb late native transport errors.
      ws.removeEventListener('open', onOpen);
      ws.removeEventListener('message', onMessage);
      ws.removeEventListener('close', onClose);
      ws.close();
    }
  }
}
