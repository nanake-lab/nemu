import type { Emotion, GazeTarget, PerformanceCue } from '@nemu/contracts';
import { isPerformableMotion } from '../fsm/slots.js';
import type { CueSchedulerOptions } from './types.js';

export class PerformanceCueScheduler {
  private readonly options?: CueSchedulerOptions | undefined;
  private readonly cueCache = new Map<string, PerformanceCue[]>();
  private activeSentenceId: string | null = null;

  constructor(options?: CueSchedulerOptions | undefined) {
    this.options = options;
  }

  cacheCues(sentenceId: string, cues: PerformanceCue[]): void {
    this.cueCache.set(sentenceId, [...cues]);
  }

  triggerCues(sentenceId: string): void {
    this.activeSentenceId = sentenceId;
    const cues = this.cueCache.get(sentenceId);
    if (!cues || cues.length === 0) return;

    for (const cue of cues) {
      switch (cue.kind) {
        case 'act': {
          if (isPerformableMotion(cue.value)) {
            this.options?.onPerformAct?.(cue.value);
          } else {
            console.warn(`Performance cue 'act' slot '${cue.value}' is not performable, ignoring.`);
          }
          break;
        }

        case 'look': {
          this.options?.onPerformLook?.(cue.value as GazeTarget);
          break;
        }

        case 'emo': {
          this.options?.onPerformEmo?.(cue.value as Emotion, cue.intensity);
          break;
        }

        default:
          // Other cues like pause/voice handled at TTS level
          break;
      }
    }
  }

  onSentenceEnd(sentenceId: string): void {
    this.cueCache.delete(sentenceId);
    if (this.activeSentenceId === sentenceId) {
      this.activeSentenceId = null;
    }
  }

  getActiveSentenceId(): string | null {
    return this.activeSentenceId;
  }

  reset(): void {
    this.cueCache.clear();
    this.activeSentenceId = null;
  }
}
