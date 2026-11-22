import { mkdirSync, readFileSync, readdirSync, renameSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { appBusEventSchema, type ApplicationEvent } from '@nemu/contracts';

export type ChatEvent = Extract<ApplicationEvent, { type: 'chat.send' }>;
const eventId = /^[0-7][0-9A-HJKMNP-TV-Z]{25}$/;

export class ChatOutbox {
  private readonly events = new Map<string, ChatEvent>();

  constructor(private readonly directory: string) {
    mkdirSync(directory, { recursive: true });
    const recovered: ChatEvent[] = [];
    for (const name of readdirSync(directory)) {
      if (name.endsWith('.tmp') && eventId.test(name.slice(0, -4))) {
        // Staging files have never been offered to the gateway.
        rmSync(join(directory, name), { force: true });
      }
      const id = name.slice(0, -5);
      if (!name.endsWith('.json') || !eventId.test(id)) continue;
      const data: unknown = JSON.parse(readFileSync(this.path(id), 'utf8'));
      const event = appBusEventSchema.parse(data);
      if (event.type !== 'chat.send' || event.source !== 'panel' || event.id !== id) {
        throw new Error('Invalid terminal outbox record');
      }
      recovered.push(event);
    }
    // The renderer timestamp preserves ordering when ULIDs share a millisecond.
    recovered.sort((a, b) => a.ts - b.ts || a.id.localeCompare(b.id));
    for (const event of recovered) this.events.set(event.id, event);
  }

  add(event: ChatEvent): void {
    if (!eventId.test(event.id)) throw new Error('Invalid chat event ID');
    const existing = this.events.get(event.id);
    if (existing) {
      if (existing.ts !== event.ts || existing.payload.text !== event.payload.text) {
        throw new Error('Conflicting chat event ID');
      }
      return;
    }
    const temporary = join(this.directory, event.id + '.tmp');
    writeFileSync(temporary, JSON.stringify(event), { flush: true });
    renameSync(temporary, this.path(event.id));
    this.events.set(event.id, event);
  }

  pending(): Iterable<ChatEvent> {
    return this.events.values();
  }

  receivedDurable(id: string): void {
    rmSync(this.path(id), { force: true });
    this.events.delete(id);
  }

  private path(id: string): string {
    return join(this.directory, id + '.json');
  }
}
