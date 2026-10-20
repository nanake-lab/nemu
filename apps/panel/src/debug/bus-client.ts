import type { ApplicationEvent } from '@nemu/contracts';
import type { DebugEventItem, EventSource } from './types.js';
import { generateUlid } from './ulid.js';

export type DebugEventListener = (item: DebugEventItem) => void;

export class DebugBusClient {
  private port: MessagePort | null = null;
  private listeners: Set<DebugEventListener> = new Set();
  private pendingOutgoing: unknown[] = [];
  private isDestroyed = false;

  constructor() {
    this.initWindowListener();
  }

  private initWindowListener(): void {
    if (typeof window === 'undefined') return;

    const onMessage = (event: MessageEvent) => {
      if (this.isDestroyed) return;
      const data = event.data as { type?: string } | null;
      if (!data || typeof data !== 'object') return;

      if (data.type === 'nemu:channel:bus' && event.ports && event.ports[0]) {
        this.setPort(event.ports[0]);
      }
    };

    window.addEventListener('message', onMessage);
  }

  public setPort(port: MessagePort): void {
    if (this.port) {
      try {
        this.port.close();
      } catch {
        // ignore
      }
    }

    this.port = port;
    this.port.onmessage = (event: MessageEvent) => {
      this.handlePortMessage(event.data);
    };

    if (typeof this.port.start === 'function') {
      this.port.start();
    }

    // Flush any pending messages
    while (this.pendingOutgoing.length > 0) {
      const msg = this.pendingOutgoing.shift();
      try {
        this.port.postMessage(msg);
      } catch (err) {
        console.error('[DebugBusClient] Failed to post pending message:', err);
      }
    }
  }

  private handlePortMessage(data: unknown): void {
    if (!data || typeof data !== 'object') return;
    const msg = data as { type?: string; event?: ApplicationEvent };

    let event: ApplicationEvent | null = null;
    if (msg.type === 'bus:event' && msg.event) {
      event = msg.event;
    } else if ('v' in msg && 'type' in msg && 'payload' in msg) {
      event = msg as unknown as ApplicationEvent;
    }

    if (event) {
      const item: DebugEventItem = {
        id: event.id ?? generateUlid(),
        ts: event.ts ?? Date.now(),
        source: (event.source as EventSource) ?? 'main',
        type: event.type,
        payload: event.payload,
        corr: event.corr,
        direction: 'in',
      };
      this.notifyListeners(item);
    }
  }

  public subscribe(listener: DebugEventListener): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notifyListeners(item: DebugEventItem): void {
    for (const listener of this.listeners) {
      try {
        listener(item);
      } catch (err) {
        console.error('[DebugBusClient] Error in listener:', err);
      }
    }
  }

  /**
   * Dispatches an event directly (used for manual triggers, tests, or outgoing commands).
   */
  public injectEvent(item: DebugEventItem): void {
    this.notifyListeners(item);
  }

  /**
   * Sends a manual stage command from the panel.
   * Records it in the event stream and forwards to the bus / port.
   */
  public sendStageCommand(type: string, payload: unknown): DebugEventItem {
    const item: DebugEventItem = {
      id: generateUlid(),
      ts: typeof performance !== 'undefined' && performance.timeOrigin
        ? performance.timeOrigin + performance.now()
        : Date.now(),
      source: 'panel',
      type,
      payload,
      direction: 'out',
    };

    // 1. Post to connected port if available
    const wireMsg = {
      type: 'bus:event',
      event: {
        v: 2,
        id: item.id,
        ts: item.ts,
        source: 'panel',
        type,
        payload,
      },
    };

    if (this.port) {
      try {
        this.port.postMessage(wireMsg);
      } catch (err) {
        console.warn('[DebugBusClient] Failed to post message to busPort:', err);
      }
    } else {
      this.pendingOutgoing.push(wireMsg);
    }

    // 2. Also notify local subscribers so it appears in the event stream viewer immediately
    this.notifyListeners(item);

    return item;
  }

  public isConnected(): boolean {
    return this.port !== null;
  }

  public destroy(): void {
    this.isDestroyed = true;
    this.listeners.clear();
    if (this.port) {
      try {
        this.port.close();
      } catch {
        // ignore
      }
      this.port = null;
    }
  }
}

// Singleton instance for the panel runtime
export const globalDebugBusClient = new DebugBusClient();
