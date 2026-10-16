import { describe, it, expect, vi } from 'vitest';
import type { PerformanceCue } from '@nemu/contracts';
import { PerformanceCueScheduler } from './cue-scheduler.js';

describe('PerformanceCueScheduler', () => {
  it('caches cues on speech begin and triggers performable cues on speech started', () => {
    const actFn = vi.fn();
    const lookFn = vi.fn();
    const emoFn = vi.fn();

    const scheduler = new PerformanceCueScheduler({
      onPerformAct: actFn,
      onPerformLook: lookFn,
      onPerformEmo: emoFn,
    });

    const cues: PerformanceCue[] = [
      { kind: 'act', value: 'wave' }, // performable
      { kind: 'look', value: 'user' },
      { kind: 'emo', value: 'happy', intensity: 0.9 },
    ];

    // 1. Sentence begins: cues cached
    scheduler.cacheCues('sent-1', cues);
    expect(actFn).not.toHaveBeenCalled();
    expect(lookFn).not.toHaveBeenCalled();

    // 2. Sentence playback starts: cues triggered
    scheduler.triggerCues('sent-1');
    expect(actFn).toHaveBeenCalledWith('wave');
    expect(lookFn).toHaveBeenCalledWith('user');
    expect(emoFn).toHaveBeenCalledWith('happy', 0.9);

    // 3. Sentence ends: state cleaned up
    scheduler.onSentenceEnd('sent-1');
    expect(scheduler.getActiveSentenceId()).toBeNull();
  });

  it('safely rejects non-performable motion slots in act cues without throwing', () => {
    const actFn = vi.fn();
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    const scheduler = new PerformanceCueScheduler({
      onPerformAct: actFn,
    });

    const cues: PerformanceCue[] = [
      { kind: 'act', value: 'idle_stand' }, // non-performable!
      { kind: 'act', value: 'fidget_yawn' }, // non-performable!
      { kind: 'act', value: 'nod' }, // performable!
    ];

    scheduler.cacheCues('sent-invalid', cues);
    scheduler.triggerCues('sent-invalid');

    // Only 'nod' should have been triggered
    expect(actFn).toHaveBeenCalledTimes(1);
    expect(actFn).toHaveBeenCalledWith('nod');
    expect(warnSpy).toHaveBeenCalledTimes(2);

    warnSpy.mockRestore();
  });
});
