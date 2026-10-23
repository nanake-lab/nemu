import {
  controlCommandSchema,
  microphoneMessageSchema,
  type MicrophoneMessage,
} from '@nemu/contracts';
export { workletSource } from './mic-worklet.js';
export interface MicHandle {
  stop(): Promise<void>;
}
export async function startMic(
  port: Pick<MessagePort, 'postMessage'>,
  workletUrl = new URL('asr-mic-worklet.js', location.href).href,
): Promise<MicHandle> {
  const stream = await navigator.mediaDevices.getUserMedia({
    audio: {
      channelCount: 1,
      echoCancellation: true,
      noiseSuppression: true,
      autoGainControl: true,
    },
    video: false,
  });
  let context: AudioContext | undefined;
  try {
    context = new AudioContext({ sampleRate: 16000 });
    await context.audioWorklet.addModule(workletUrl);
    const source = context.createMediaStreamSource(stream);
    const node = new AudioWorkletNode(context, 'nemu-mic');
    let seq = 0;
    node.port.onmessage = (e: MessageEvent<unknown>) => {
      const msg: MicrophoneMessage = {
        v: 2,
        seq: seq++,
        pcm: new Int16Array(e.data as Int16Array),
      };
      if (microphoneMessageSchema.safeParse(msg).success) port.postMessage(msg);
    };
    // Worklet must be pulled by an output; zero gain prevents microphone monitoring.
    const mute = context.createGain();
    mute.gain.value = 0;
    source.connect(node).connect(mute).connect(context.destination);
    await context.resume();
    const activeContext = context;
    return {
      stop: async () => {
        node.port.onmessage = null;
        node.disconnect();
        source.disconnect();
        mute.disconnect();
        stream.getTracks().forEach((t) => t.stop());
        await activeContext.close();
      },
    };
  } catch (e) {
    stream.getTracks().forEach((t) => t.stop());
    await context?.close();
    throw e;
  }
}
/** Serialize permission prompts and discard a late getUserMedia result after disable/reload. */
export class MicController {
  private generation = 0;
  private frames = 0;
  status(): { active: boolean; frames: number } {
    return { active: this.handle !== undefined, frames: this.frames };
  }
  private handle: MicHandle | undefined;
  private pending: Promise<void> = Promise.resolve();
  constructor(
    private port: Pick<MessagePort, 'postMessage'>,
    private open = startMic,
  ) {}
  set(enabled: boolean): Promise<void> {
    const generation = ++this.generation;
    this.pending = this.pending
      .catch(() => {})
      .then(async () => {
        await this.handle?.stop();
        this.handle = undefined;
        if (!enabled || generation !== this.generation) return;
        this.frames = 0;
        const handle = await this.open({
          postMessage: (message: unknown) => {
            if (generation === this.generation) {
              this.frames++;
              this.port.postMessage(message);
            }
          },
        });
        if (generation !== this.generation) await handle.stop();
        else this.handle = handle;
      });
    return this.pending;
  }
  command(raw: unknown): Promise<void> {
    const parsed = controlCommandSchema.safeParse(raw);
    return parsed.success && parsed.data.type === 'stage.mic.set'
      ? this.set(parsed.data.payload.enabled)
      : Promise.resolve();
  }
}
