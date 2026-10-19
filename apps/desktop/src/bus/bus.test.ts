import { describe, it, expect, vi } from 'vitest';
import { z } from 'zod';
import type { ApplicationEvent } from '@nemu/contracts';
import { RingBuffer, generateUlid, AppBus, type BusPortLike } from './index.js';

describe('RingBuffer', () => {
  it('stores items up to capacity and drops oldest on overflow (FIFO)', () => {
    const rb = new RingBuffer<number>(3);
    expect(rb.length).toBe(0);

    rb.push(1);
    rb.push(2);
    rb.push(3);
    expect(rb.length).toBe(3);
    expect(rb.toArray()).toEqual([1, 2, 3]);

    // Push 4th: 1 should be dropped
    rb.push(4);
    expect(rb.length).toBe(3);
    expect(rb.toArray()).toEqual([2, 3, 4]);
    expect(rb.get(0)).toBe(2);
    expect(rb.get(2)).toBe(4);
    expect(rb.getLatest()).toBe(4);

    // Push 5th: 2 should be dropped
    rb.push(5);
    expect(rb.toArray()).toEqual([3, 4, 5]);

    rb.clear();
    expect(rb.length).toBe(0);
    expect(rb.toArray()).toEqual([]);
  });

  it('handles custom 5000 capacity', () => {
    const rb = new RingBuffer<number>(5000);
    for (let i = 0; i < 5500; i++) {
      rb.push(i);
    }
    expect(rb.length).toBe(5000);
    expect(rb.get(0)).toBe(500);
    expect(rb.getLatest()).toBe(5499);
  });
});

describe('generateUlid', () => {
  it('generates a 26-character string matching zod ulid validator', () => {
    const ulidSchema = z.ulid();
    for (let i = 0; i < 20; i++) {
      const id = generateUlid();
      expect(id).toHaveLength(26);
      expect(ulidSchema.safeParse(id).success).toBe(true);
    }
  });
});

describe('AppBus', () => {
  class MockPort implements BusPortLike {
    public sent: unknown[] = [];
    public onmessage: ((event: { data: unknown }) => void) | null = null;

    postMessage(message: unknown): void {
      this.sent.push(message);
    }

    emit(data: unknown): void {
      if (this.onmessage) {
        this.onmessage({ data });
      }
    }
  }

  it('publishes and subscribes to typed events', () => {
    const bus = new AppBus({ enableValidation: true });
    const vadReceived: unknown[] = [];
    const allReceived: unknown[] = [];

    const unsubVad = bus.subscribe('voice.vad', (e) => vadReceived.push(e));
    const unsubAll = bus.subscribe('*', (e) => allReceived.push(e));

    const event = bus.createEvent('voice', 'voice.vad', { state: 'speechStart' });
    const published = bus.publish(event);

    expect(published).toBe(true);
    expect(vadReceived).toHaveLength(1);
    expect(allReceived).toHaveLength(1);
    expect(bus.getHistory()).toHaveLength(1);

    unsubVad();
    bus.publish(bus.createEvent('voice', 'voice.vad', { state: 'speechEnd' }));
    expect(vadReceived).toHaveLength(1);
    expect(allReceived).toHaveLength(2);

    unsubAll();
  });

  it('rejects invalid events when validation is enabled', () => {
    const bus = new AppBus({ enableValidation: true });
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    // Invalid payload
    const invalidEvent = {
      v: 2 as const,
      id: generateUlid(),
      ts: Date.now(),
      source: 'main' as const,
      type: 'chat.send' as const,
      payload: { invalidField: 123 } as unknown as { text: string },
    };

    const published = bus.publish(invalidEvent as unknown as ApplicationEvent);
    expect(published).toBe(false);
    expect(bus.getHistory()).toHaveLength(0);
    expect(warnSpy).toHaveBeenCalled();
    warnSpy.mockRestore();
  });

  it('routes events to and from connected MessagePorts without echo', () => {
    const bus = new AppBus({ enableValidation: true });
    const portStage = new MockPort();
    const portPanel = new MockPort();

    bus.connectPort('stage', portStage);
    bus.connectPort('panel', portPanel);

    // Event originating from portStage should be forwarded to portPanel, but not echoed back to portStage
    const chatEvent = bus.createEvent('panel', 'chat.send', { text: 'Hello Nemu' });
    portStage.emit({ type: 'bus:event', event: chatEvent });

    expect(portPanel.sent).toHaveLength(1);
    expect(portPanel.sent[0]).toEqual({ type: 'bus:event', event: chatEvent });
    expect(portStage.sent).toHaveLength(0);

    // Event published locally on main bus should be forwarded to both ports
    const dndEvent = bus.createEvent('main', 'desktop.dnd', { state: 'busy' });
    bus.publish(dndEvent);

    expect(portPanel.sent).toHaveLength(2);
    expect(portStage.sent).toHaveLength(1);
  });
});
