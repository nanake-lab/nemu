import {
  type ApplicationEvent,
  type AppBusPayloads,
  appBusEventSchema,
  forwardedAppBusTypes,
} from '@nemu/contracts';
import { RingBuffer } from './ring-buffer.js';
import { generateUlid } from './ulid.js';

export interface BusPortLike {
  postMessage(message: unknown): void;
  start?(): void;
  close?(): void;
  on?(event: 'message', listener: (event: { data: unknown }) => void): void;
  onmessage?: ((event: { data: unknown }) => void) | null;
  addEventListener?(type: 'message', listener: (event: { data: unknown }) => void): void;
  removeEventListener?(type: 'message', listener: (event: { data: unknown }) => void): void;
}

export type BusEventHandler<T extends ApplicationEvent = ApplicationEvent> = (event: T) => void;

export class AppBus {
  private ringBuffer: RingBuffer<ApplicationEvent>;
  private listeners: Map<string, Set<BusEventHandler>> = new Map();
  private wildcardListeners: Set<BusEventHandler> = new Set();
  private ports: Map<string, BusPortLike> = new Map();
  public enableValidation: boolean;

  constructor(options?: { ringCapacity?: number; enableValidation?: boolean }) {
    this.ringBuffer = new RingBuffer<ApplicationEvent>(options?.ringCapacity ?? 5000);
    this.enableValidation = options?.enableValidation ?? (process.env.NODE_ENV !== 'production');
  }

  public createEvent<K extends keyof AppBusPayloads>(
    source: 'main' | 'stage' | 'panel' | 'voice',
    type: K,
    payload: AppBusPayloads[K],
    corr?: string,
  ): ApplicationEvent {
    const ts = typeof performance !== 'undefined' && performance.timeOrigin
      ? performance.timeOrigin + performance.now()
      : Date.now();

    const event = {
      v: 2 as const,
      id: generateUlid(),
      ts,
      source,
      type,
      payload,
      ...(corr ? { corr } : {}),
    } as ApplicationEvent;

    return event;
  }

  public publish(event: ApplicationEvent, originPortId?: string): boolean {
    if (this.enableValidation) {
      const parsed = appBusEventSchema.safeParse(event);
      if (!parsed.success) {
        console.warn(`[AppBus] Invalid event for type '${event.type}':`, parsed.error.issues);
        return false;
      }
    }

    // Record in ring buffer (debug-only FIFO history)
    this.ringBuffer.push(event);

    // Dispatch to local type listeners
    const handlers = this.listeners.get(event.type);
    if (handlers) {
      for (const handler of handlers) {
        try {
          handler(event);
        } catch (err) {
          console.error(`[AppBus] Error in handler for '${event.type}':`, err);
        }
      }
    }

    // Dispatch to wildcard listeners
    for (const handler of this.wildcardListeners) {
      try {
        handler(event);
      } catch (err) {
        console.error(`[AppBus] Error in wildcard handler for '${event.type}':`, err);
      }
    }

    // Route to connected ports (except the originating port)
    for (const [portId, port] of this.ports.entries()) {
      if (portId !== originPortId) {
        try {
          port.postMessage({ type: 'bus:event', event });
        } catch (err) {
          console.error(`[AppBus] Failed to forward event to port '${portId}':`, err);
        }
      }
    }

    return true;
  }

  public subscribe<T extends ApplicationEvent = ApplicationEvent>(
    typeOrTypes: string | string[],
    handler: BusEventHandler<T>,
  ): () => void {
    const types = Array.isArray(typeOrTypes) ? typeOrTypes : [typeOrTypes];

    for (const type of types) {
      if (type === '*') {
        this.wildcardListeners.add(handler as BusEventHandler);
      } else {
        let set = this.listeners.get(type);
        if (!set) {
          set = new Set();
          this.listeners.set(type, set);
        }
        set.add(handler as BusEventHandler);
      }
    }

    return () => {
      for (const type of types) {
        if (type === '*') {
          this.wildcardListeners.delete(handler as BusEventHandler);
        } else {
          const set = this.listeners.get(type);
          if (set) {
            set.delete(handler as BusEventHandler);
            if (set.size === 0) {
              this.listeners.delete(type);
            }
          }
        }
      }
    };
  }

  public connectPort(portId: string, port: BusPortLike): () => void {
    this.ports.set(portId, port);

    const onMessage = (event: { data: unknown }) => {
      const data = event.data;
      if (!data || typeof data !== 'object') return;
      const msg = data as { type?: string; event?: ApplicationEvent };
      const rawEvent =
        msg.type === 'bus:event' && msg.event
          ? msg.event
          : 'v' in msg && 'type' in msg && 'payload' in msg
            ? (msg as unknown as ApplicationEvent)
            : null;
      if (!rawEvent) return;

      // Restrict source and permissions according to port identity
      if (portId === 'stage') {
        if ((forwardedAppBusTypes as readonly string[]).includes(rawEvent.type)) {
          console.warn(`[AppBus] Dropped unauthorized forwarded event type '${rawEvent.type}' from stage port.`);
          return;
        }
        rawEvent.source = 'stage';
      } else if (portId === 'panel') {
        rawEvent.source = 'panel';
      }

      this.publish(rawEvent, portId);
    };

    if (typeof port.addEventListener === 'function') {
      port.addEventListener('message', onMessage);
    } else if (typeof port.on === 'function') {
      port.on('message', onMessage);
    } else {
      port.onmessage = onMessage;
    }

    if (typeof port.start === 'function') {
      port.start();
    }

    return () => {
      this.disconnectPort(portId);
    };
  }

  public disconnectPort(portId: string): void {
    const port = this.ports.get(portId);
    if (port) {
      this.ports.delete(portId);
      if (typeof port.close === 'function') {
        try {
          port.close();
        } catch {
          // ignore
        }
      }
    }
  }

  public getHistory(): ApplicationEvent[] {
    return this.ringBuffer.toArray();
  }

  public clearHistory(): void {
    this.ringBuffer.clear();
  }
}
