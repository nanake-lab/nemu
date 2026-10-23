import type { AppBusPayloads, Lang } from '@nemu/contracts';
export type AsrResult = Pick<AppBusPayloads['voice.recognized'], 'text' | 'lang' | 'confidence'> & {
  metadata?: { utteranceKey: string; durationMs: number; vadEnd: number; asrDone: number };
};
export interface AsrSession {
  feed(pcm: Int16Array): void;
  onVad(cb: (state: AppBusPayloads['voice.vad']['state']) => void): void;
  onResult(cb: (result: AsrResult) => void): void;
  close(): void;
}
export interface AsrProvider {
  id: string;
  start(options: { lang: 'auto' | Lang; mode: 'pushToTalk' | 'continuous' }): AsrSession;
}
export class CloudAsrProvider implements AsrProvider {
  readonly id = 'cloud-unconfigured';
  start(): AsrSession {
    throw new Error('Cloud ASR adapter is not configured; audio remains local');
  }
}
export function normalizeLanguage(raw: string, text: string): { lang: Lang; mapped: boolean } {
  const code = raw.replace(/<\||\|>/g, '');
  if (code === 'zh' || code === 'en' || code === 'ja') return { lang: code, mapped: false };
  if (code === 'yue') return { lang: 'zh', mapped: true };
  // Unsupported languages have no exact Lang. Use script proximity and expose mapping separately.
  return {
    lang: /[\u3040-\u30ff]/u.test(text) ? 'ja' : /\p{Script=Han}/u.test(text) ? 'zh' : 'en',
    mapped: true,
  };
}
