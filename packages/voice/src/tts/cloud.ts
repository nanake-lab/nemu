import { ttsRequestSchema, type Lang, type TtsProvider, type TtsRequest } from '@nemu/contracts';

export interface CloudTtsOptions {
  baseUrl: string;
  model: string;
  /** Supplied by the caller after safeStorage decryption; never persisted here. */
  apiKey: () => Promise<string>;
  voices: Record<
    string,
    {
      providerVoice: string;
      instructions: Partial<Record<Lang, Record<string, string>>>;
    }
  >;
  supportsInstructions: boolean;
  fetcher?: typeof fetch;
}

/** OpenAI-compatible audio/speech, response_format=pcm: 24 kHz signed int16 LE. */
export class CloudTtsProvider implements TtsProvider {
  readonly id = 'openai-compatible-tts';
  constructor(private readonly options: CloudTtsOptions) {}

  async *synthesize(input: TtsRequest, signal: AbortSignal): AsyncIterable<Float32Array> {
    const req = ttsRequestSchema.parse(input);
    if (signal.aborted) return;
    const voice = this.options.voices[req.voiceId];
    if (!voice) throw new Error('Cloud voice is not registered');
    const instruction = voice.instructions[req.lang]?.[req.style];
    if (this.options.supportsInstructions && instruction === undefined) {
      throw new Error('Cloud style instruction is not registered');
    }
    const endpoint = new URL('audio/speech', this.options.baseUrl.replace(/\/?$/, '/'));
    if (
      endpoint.protocol !== 'https:' &&
      endpoint.hostname !== '127.0.0.1' &&
      endpoint.hostname !== 'localhost'
    ) {
      throw new Error('Cloud TTS requires HTTPS');
    }
    const key = await this.options.apiKey();
    if (signal.aborted) return;
    const response = await (this.options.fetcher ?? fetch)(endpoint, {
      method: 'POST',
      signal,
      headers: { Authorization: `Bearer ${key}`, 'Content-Type': 'application/json' },
      body: JSON.stringify({
        model: this.options.model,
        input: req.text,
        voice: voice.providerVoice,
        response_format: 'pcm',
        ...(this.options.supportsInstructions ? { instructions: instruction } : {}),
      }),
    });
    if (!response.ok || !response.body) {
      // Response bodies can contain request content or credentials: do not log them.
      throw new Error(`Cloud TTS failed: ${response.status}`);
    }
    const reader = response.body.getReader();
    const abort = (): void => {
      void reader.cancel().catch(() => undefined);
    };
    signal.addEventListener('abort', abort, { once: true });
    let carry: number | undefined;
    try {
      while (!signal.aborted) {
        const { done, value } = await reader.read();
        if (signal.aborted) return;
        if (done) break;
        const bytes = carry === undefined ? value : Uint8Array.from([carry, ...value]);
        const length = bytes.byteLength - (bytes.byteLength % 2);
        carry = length < bytes.byteLength ? bytes[length] : undefined;
        const view = new DataView(bytes.buffer, bytes.byteOffset, length);
        const pcm = new Float32Array(length / 2);
        for (let i = 0; i < pcm.length; i++) pcm[i] = view.getInt16(i * 2, true) / 32768;
        if (pcm.length) yield pcm;
      }
      if (!signal.aborted && carry !== undefined) throw new Error('Truncated cloud PCM');
    } finally {
      signal.removeEventListener('abort', abort);
      await reader.cancel().catch(() => undefined);
      reader.releaseLock();
    }
  }
}
