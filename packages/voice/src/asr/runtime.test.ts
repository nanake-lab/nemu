import { describe, expect, it, vi } from 'vitest';
import type { ApplicationEvent, MicrophoneMessage } from '@nemu/contracts';
const fixture = vi.hoisted(() => ({ text: 'hello', lang: 'en', decodes: 0 }));
vi.mock('sherpa-onnx-node', () => ({
  default: {
    OfflineRecognizer: class {
      createStream() {
        return { acceptWaveform: () => {} };
      }
      async decodeAsync() {
        fixture.decodes++;
        return { text: fixture.text, lang: fixture.lang };
      }
    },
    Vad: class {
      private detected = false;
      private count = 0;
      private minimum: number;
      private segments: Float32Array[] = [];
      constructor(config: { sileroVad: { minSpeechDuration: number } }) {
        this.minimum = config.sileroVad.minSpeechDuration >= 0.2 ? 3200 : 1;
      }
      acceptWaveform(samples: Float32Array) {
        const speech = (samples[0] ?? 0) > 0;
        if (speech) this.count += samples.length;
        else {
          if (this.detected) this.segments.push(new Float32Array(this.count));
          this.count = 0;
        }
        this.detected = speech && this.count >= this.minimum;
      }
      isDetected() {
        return this.detected;
      }
      isEmpty() {
        return this.segments.length === 0;
      }
      front() {
        return { samples: this.segments[0] };
      }
      pop() {
        this.segments.shift();
      }
      flush() {
        if (this.count) this.segments.push(new Float32Array(this.count));
        this.count = 0;
      }
      reset() {
        this.detected = false;
        this.segments = [];
        this.count = 0;
      }
    },
  },
}));
import { VoiceRuntime, type VoicePort } from './runtime.js';
class Port implements VoicePort {
  listener: ((e: { data: unknown }) => void) | undefined;
  on(_event: 'message', cb: (e: { data: unknown }) => void) {
    this.listener = cb;
  }
  start() {}
  close() {
    this.listener = undefined;
  }
  send(data: unknown) {
    this.listener?.({ data });
  }
}
const command = (enabled: boolean) => ({
  v: 2,
  id: 'mic',
  type: 'stage.mic.set',
  payload: { enabled, mode: 'pushToTalk' },
});
const packet = (seq: number, speech = true): MicrophoneMessage => ({
  v: 2,
  seq,
  pcm: new Int16Array(320).fill(speech ? 100 : 0),
});
const settle = async () => {
  await new Promise((r) => setTimeout(r, 0));
};
describe('P4-01 voice runtime protocol', () => {
  it('validates packets and emits keyed results/marks only after speech', async () => {
    const events: ApplicationEvent[] = [];
    const runtime = new VoiceRuntime('fixture', (e) => events.push(e));
    const mic = new Port();
    runtime.attach(mic, new Port());
    mic.send(packet(0));
    expect(events).toEqual([]);
    runtime.command(command(true));
    mic.send({ v: 1, seq: 0, pcm: [] });
    mic.send(packet(0));
    mic.send(packet(1, false));
    await settle();
    expect(events.map((e) => e.type)).toEqual([
      'voice.vad',
      'voice.vad',
      'metrics.latency',
      'voice.recognized',
    ]);
    const result = events.find((e) => e.type === 'voice.recognized');
    const metrics = events.find((e) => e.type === 'metrics.latency');
    expect(result?.payload).toMatchObject({ text: 'hello', lang: 'en', durationMs: 20 });
    expect(result?.payload.utteranceKey).toBe(metrics?.payload.utteranceKey);
    runtime.close();
  });
  it('requires 200 ms confirmed speech during playback; once per utterance', () => {
    const events: ApplicationEvent[] = [];
    const runtime = new VoiceRuntime('fixture', (e) => events.push(e));
    const mic = new Port(),
      audio = new Port();
    runtime.attach(mic, audio);
    runtime.command(command(true));
    audio.send({
      v: 2,
      id: 'audio',
      type: 'stage.speech.started',
      payload: { sentenceId: 's', at: 0 },
    });
    for (let i = 0; i < 9; i++) mic.send(packet(i));
    expect(events.filter((e) => e.type === 'voice.bargeIn')).toHaveLength(0);
    for (let i = 9; i < 20; i++) mic.send(packet(i));
    expect(events.filter((e) => e.type === 'voice.bargeIn')).toHaveLength(1);
    runtime.close();
  });
  it('flushes final PTT speech on disable', async () => {
    const events: ApplicationEvent[] = [];
    const runtime = new VoiceRuntime('fixture', (e) => events.push(e));
    const mic = new Port();
    runtime.attach(mic, new Port());
    runtime.command(command(true));
    mic.send(packet(0));
    runtime.command(command(false));
    await settle();
    expect(events.filter((e) => e.type === 'voice.recognized')).toHaveLength(1);
    runtime.close();
  });
  it('rejects sequence gaps without joining two unrelated PCM ranges', async () => {
    const events: ApplicationEvent[] = [];
    const runtime = new VoiceRuntime('fixture', (e) => events.push(e));
    const mic = new Port();
    runtime.attach(mic, new Port());
    runtime.command(command(true));
    mic.send(packet(0));
    mic.send(packet(2));
    expect(events.find((e) => e.type === 'stage.error')?.payload.code).toBe('ASR_PCM_GAP');
    runtime.close();
    await settle();
    expect(events.filter((e) => e.type === 'voice.recognized')).toHaveLength(0);
  });
  it('does not publish old recognition after a renderer reload', async () => {
    const events: ApplicationEvent[] = [];
    const runtime = new VoiceRuntime('fixture', (e) => events.push(e));
    const mic = new Port();
    runtime.attach(mic, new Port());
    runtime.command(command(true));
    mic.send(packet(0));
    runtime.attach(new Port(), new Port());
    await settle();
    expect(events.filter((e) => e.type === 'voice.recognized')).toHaveLength(0);
    runtime.close();
  });
});
