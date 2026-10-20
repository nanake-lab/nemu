import { describe, it, expect, vi } from 'vitest';
import { DebugBusClient } from './bus-client.js';
import { generateUlid } from './ulid.js';

describe('bus-client and ulid', () => {
  it('generates valid 26-char Crockford Base32 ULIDs', () => {
    const ulidRegex = /^[0-7][0-9A-HJKMNP-TV-Z]{25}$/i;
    for (let i = 0; i < 50; i++) {
      const id = generateUlid();
      expect(id).toHaveLength(26);
      expect(ulidRegex.test(id), `ULID ${id} should match Crockford format`).toBe(true);
    }
  });

  it('notifies subscribers when events are injected', () => {
    const client = new DebugBusClient();
    const listener = vi.fn();
    const unsub = client.subscribe(listener);

    const testItem = {
      id: generateUlid(),
      ts: Date.now(),
      source: 'main' as const,
      type: 'core.runState',
      payload: {
        states: ['RUNNING'],
        speechMode: 'A_LOCAL_STRICT' as const,
        strict: true,
        lineage: 'LIFE' as const,
        autonomy: 'active' as const,
        reasons: [],
      },
    };

    client.injectEvent(testItem);
    expect(listener).toHaveBeenCalledWith(testItem);

    unsub();
    client.injectEvent(testItem);
    expect(listener).toHaveBeenCalledTimes(1);
    client.destroy();
  });

  it('dispatches manual stage commands and records them in event stream', () => {
    const client = new DebugBusClient();
    const listener = vi.fn();
    client.subscribe(listener);

    const cuePayload = {
      cue: {
        kind: 'emo',
        value: 'happy',
        intensity: 0.8,
      },
    };

    const item = client.sendStageCommand('stage.perform', cuePayload);
    expect(item.type).toBe('stage.perform');
    expect(item.source).toBe('panel');
    expect(item.payload).toEqual(cuePayload);
    expect(item.direction).toBe('out');

    expect(listener).toHaveBeenCalledWith(item);
    client.destroy();
  });

  it('buffers and flushes outgoing messages when port is connected', () => {
    const client = new DebugBusClient();
    client.sendStageCommand('stage.perform', { cue: { kind: 'emo', value: 'happy' } });

    const postMessageSpy = vi.fn();
    const mockPort = {
      postMessage: postMessageSpy,
      start: vi.fn(),
      close: vi.fn(),
    } as unknown as MessagePort;

    client.setPort(mockPort);
    expect(postMessageSpy).toHaveBeenCalledTimes(1);
    expect(postMessageSpy.mock.calls[0]?.[0]?.type).toBe('bus:event');
    expect(postMessageSpy.mock.calls[0]?.[0]?.event?.type).toBe('stage.perform');
    client.destroy();
  });
});
