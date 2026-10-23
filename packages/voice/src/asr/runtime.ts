import { randomBytes } from 'node:crypto';
import {
  microphoneMessageSchema,
  controlCommandSchema,
  audioEventSchema,
  appBusEventSchema,
  type ApplicationEvent,
  type AppBusPayloads,
} from '@nemu/contracts';
import { SenseVoiceProvider, epochNow } from './sensevoice.js';
import type { AsrSession } from './provider.js';
import { cacheDirectory, verifyModels } from './models.js';
export interface VoicePort {
  on(event: 'message', cb: (e: { data: unknown }) => void): void;
  start(): void;
  close(): void;
}
export class VoiceRuntime {
  private session: AsrSession | undefined;
  private ports: VoicePort[] = [];
  private generation = 0;
  private provider: SenseVoiceProvider;
  private seq = -1;
  constructor(
    directory: string,
    private emit: (event: ApplicationEvent) => void,
  ) {
    this.provider = new SenseVoiceProvider(directory, {
      onBargeIn: () => this.publish('voice.bargeIn', {}),
      onMappedLanguage: (raw, lang) =>
        this.publish('stage.error', {
          code: 'ASR_LANGUAGE_MAPPED',
          message: `Unsupported language ${raw} mapped to ${lang}`,
        }),
      onError: (reason) =>
        this.publish('stage.error', {
          code: reason,
          message: 'ASR session failed; microphone must be restarted',
        }),
    });
  }
  private publish<K extends keyof AppBusPayloads>(type: K, payload: AppBusPayloads[K]): void {
    // 128-bit random ULID-compatible source key; timestamp prefix uses shared epoch clock.
    const alphabet = '0123456789ABCDEFGHJKMNPQRSTVWXYZ';
    let value =
      (BigInt(Math.floor(epochNow())) << 80n) | BigInt(`0x${randomBytes(10).toString('hex')}`);
    let id = '';
    for (let i = 0; i < 26; i++) {
      id = alphabet[Number(value & 31n)] + id;
      value >>= 5n;
    }
    const event = appBusEventSchema.parse({
      v: 2,
      id,
      ts: epochNow(),
      source: 'voice',
      type,
      payload,
    });
    this.emit(event);
  }
  command(raw: unknown): void {
    const parsed = controlCommandSchema.safeParse(raw);
    if (!parsed.success || parsed.data.type !== 'stage.mic.set') return;
    this.session?.close();
    this.session = undefined;
    this.seq = -1;
    if (!parsed.data.payload.enabled) return;
    const mode = parsed.data.payload.mode;
    this.session = this.provider.start({ lang: 'auto', mode: parsed.data.payload.mode });
    const generation = ++this.generation;
    this.session.onVad((state) => {
      if (generation === this.generation) this.publish('voice.vad', { state });
    });
    this.session.onResult((result) => {
      if (generation !== this.generation || !result.metadata) return;
      const { utteranceKey, durationMs, vadEnd, asrDone } = result.metadata;
      if (
        process.env.NEMU_ASR_WAKE_WORD === '1' &&
        mode === 'continuous' &&
        !/\bnemu\b/i.test(result.text)
      )
        return;
      this.publish('metrics.latency', { utteranceKey, marks: { vadEnd, asrDone } });
      this.publish('voice.recognized', {
        utteranceKey,
        durationMs,
        text: result.text,
        lang: result.lang,
        ...(result.confidence === undefined ? {} : { confidence: result.confidence }),
      });
    });
  }
  attach(mic: VoicePort, audio: VoicePort): void {
    this.provider.resetPlayback();
    this.generation++;
    this.ports.forEach((p) => p.close());
    this.ports = [mic, audio];
    this.session?.close();
    this.session = undefined;
    this.seq = -1;
    mic.on('message', (e) => {
      const parsed = microphoneMessageSchema.safeParse(e.data);
      if (!parsed.success || !this.session) return;
      if (this.seq >= 0 && parsed.data.seq !== this.seq + 1) {
        this.generation++;
        this.session.close();
        this.session = undefined;
        this.publish('stage.error', {
          code: 'ASR_PCM_GAP',
          message: 'Microphone sequence gap; restart microphone',
        });
        return;
      }
      this.seq = parsed.data.seq;
      this.session.feed(parsed.data.pcm);
    });
    mic.start();
    audio.on('message', (e) => {
      const parsed = audioEventSchema.safeParse(e.data);
      if (!parsed.success) return;
      if (parsed.data.type === 'stage.speech.started')
        this.provider.setPlayback(parsed.data.payload.sentenceId, true);
      if (parsed.data.type === 'stage.speech.done')
        this.provider.setPlayback(parsed.data.payload.sentenceId, false);
    });
    audio.start();
  }
  close(): void {
    this.generation++;
    this.session?.close();
    this.session = undefined;
    this.ports.forEach((p) => p.close());
    this.ports = [];
  }
}
export async function createVoiceRuntime(
  emit: (event: ApplicationEvent) => void,
  lock?: string,
): Promise<VoiceRuntime> {
  await verifyModels(cacheDirectory(), lock);
  return new VoiceRuntime(cacheDirectory(), emit);
}
