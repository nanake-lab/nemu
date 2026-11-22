import { mkdtempSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { afterEach, expect, test, vi } from 'vitest';
import { EchoCore } from '@nemu/contracts';
import { AppBus } from '../../../apps/desktop/src/bus/bus.js';
import { bindTerminal } from '../../../apps/desktop/src/terminal/bridge.js';
import legal from '../../../core/contracts/golden/legal.json' with { type: 'json' };
import { scriptedChunk, scriptedCore, scriptApproval } from '../test/scripted-core.js';
import { decodeFrame, encodeFrame, FrameReader, frameLimits } from './framing.js';
import { ChatOutbox, TerminalClient, type ChatEvent } from './index.js';

const paths: string[] = [];
function temp() {
  const directory = mkdtempSync(join(tmpdir(), 'nemu-p6-chat-'));
  paths.push(directory);
  return directory;
}
afterEach(() => {
  for (const path of paths.splice(0)) rmSync(path, { recursive: true, force: true });
});

test('Rust golden protobuf matches terminal framing byte for byte across split reads', () => {
  for (const name of [
    'TerminalAuthenticate',
    'TerminalAuthResult',
    'TerminalRequest',
    'TerminalResponse',
    'OutputChunk',
    'OutputEnd',
    'OutputCancel',
  ] as const) {
    const vector = legal.find((v) => v.request.name === name && v.expected.protobuf);
    expect(vector).toBeDefined();
    const bytes = Buffer.from(vector!.expected.protobuf!, 'hex');
    const message = decodeFrame(name, bytes);
    const frame = encodeFrame(name, message);
    expect(frame.subarray(4)).toEqual(bytes);
    expect(frame.readUInt32LE()).toBe(bytes.length);
    const reader = new FrameReader();
    for (let i = 0; i < frame.length - 1; i++) {
      expect([...reader.push(frame.subarray(i, i + 1))]).toHaveLength(0);
    }
    expect([...reader.push(frame.subarray(-1))]).toEqual([bytes]);
  }
  for (const length of [0, frameLimits.max_bytes + 1]) {
    const header = Buffer.alloc(4);
    header.writeUInt32LE(length);
    expect(() => [...new FrameReader().push(header)]).toThrow();
  }
});

test('outbox persists only unacknowledged chat events and deduplicates by event ID', () => {
  const path = temp();
  const bus = new AppBus();
  const event = bus.createEvent('panel', 'chat.send', { text: 'synthetic' }) as ChatEvent;
  const outbox = new ChatOutbox(path);
  outbox.add(event);
  outbox.add(event);
  expect(readdirSync(path)).toEqual([event.id + '.json']);
  const reopened = new ChatOutbox(path);
  expect([...reopened.pending()]).toEqual([event]);
  expect(() => reopened.add({ ...event, payload: { text: 'conflict' } })).toThrow();
  reopened.receivedDurable(event.id);
  expect(readdirSync(path)).toEqual([]);
  expect([...new ChatOutbox(path).pending()]).toEqual([]);
});

test('live pipe reauthenticates and replays the original observation after a lost durable reply', async () => {
  const path = temp();
  const core = await scriptedCore(path, { dropFirstReceipt: true });
  const status = vi.fn<(reason: string | null, disconnected?: boolean) => void>();
  const terminal = new TerminalClient(new ChatOutbox(join(path, 'outbox')), status, core.identity);
  const bus = new AppBus();
  const bridge = bindTerminal(bus, terminal, (reason) => status(reason));
  try {
    const event = bus.createEvent('panel', 'chat.send', {
      text: 'สวัสดี，こんにちは 🌿',
    }) as ChatEvent;
    bus.publish(event);
    terminal.start();
    await vi.waitFor(() => expect(readdirSync(join(path, 'outbox'))).toEqual([]), {
      timeout: 4000,
    });
    expect(core.input).toHaveLength(2);
    expect(core.input[0]).toEqual(core.input[1]);
    expect(core.messages.get(event.id)).toBe(event.payload.text);
    expect(core.messages.size).toBe(1);
    expect(core.failures).toEqual([]);
    expect(status).toHaveBeenCalledWith(expect.any(String), true);
    expect(core.timestamps.authenticated).toHaveLength(2);
    const elapsed = core.timestamps.durable[0]! - core.timestamps.disconnected[0]!;
    expect(elapsed).toBeGreaterThan(0);
    expect(elapsed).toBeLessThan(3000);
    core.disconnect();
    await vi.waitFor(() => expect(core.timestamps.authenticated).toHaveLength(3), {
      timeout: 4000,
    });
    expect(core.input).toHaveLength(2);
    expect(readdirSync(join(path, 'outbox'))).toEqual([]);
    // Synthetic timings only. No token or transcript goes to the test log.
    process.stdout.write(
      'P6-02 fake core disconnect to durable retry: ' + elapsed.toFixed(1) + ' ms\n',
    );
  } finally {
    bridge.close();
    terminal.stop();
    await core.close();
  }
});

test('bridge publishes only received chunks, stops at the prefix, and pairs approvals by digest', async () => {
  const path = temp();
  const core = await scriptedCore(path);
  const bus = new AppBus();
  const status = vi.fn<(reason: string | null) => void>();
  const terminal = new TerminalClient(new ChatOutbox(join(path, 'outbox')), status, core.identity);
  const bridge = bindTerminal(bus, terminal, status);
  try {
    terminal.start();
    await vi.waitFor(() => expect(status).toHaveBeenCalledWith(null));
    bridge.chunk(scriptedChunk('First ', 0n));
    bridge.chunk(scriptedChunk('second.', 1n));
    bridge.chunk(scriptedChunk('duplicate', 1n));
    await bridge.cancel({ schema_version: 1, output_id: 'scripted-output' });
    bridge.chunk(scriptedChunk('late tail', 2n));
    bridge.end({ schema_version: 1, output_id: 'scripted-output', reason: 2 });
    bridge.end({ schema_version: 1, output_id: 'failed-turn', reason: 3 });
    bridge.runState({
      schema_version: 1,
      states: [6],
      speech_mode: 2,
      strict: false,
      autonomy: 2,
      reasons: ['IRL_FAILED'],
      lineage: 1,
    });
    bridge.approval(scriptApproval);
    bus.publish(
      bus.createEvent('panel', 'approval.decision.v2', {
        approvalId: scriptApproval.approval_id,
        decision: 'deny',
        payloadDigest: new Uint8Array(32),
      }),
    );
    await new Promise((resolve) => setImmediate(resolve));
    expect(core.controls).toHaveLength(1);
    bus.publish(
      bus.createEvent('panel', 'approval.decision.v2', {
        approvalId: scriptApproval.approval_id,
        decision: 'deny',
        payloadDigest: Uint8Array.from(scriptApproval.payload_digest),
      }),
    );
    await vi.waitFor(() => expect(core.controls).toHaveLength(2));
    expect(core.controls[0]?.cancel?.output_id).toBe('scripted-output');
    expect(core.controls[1]?.approval?.decision).toBe(EchoCore.schema.enums.ApprovalChoice!.DENY);
    expect(core.controls[1]?.approval?.payload_digest).toEqual(scriptApproval.payload_digest);
    const events = bus.getHistory();
    expect(events.flatMap((e) => (e.type === 'output.chunk' ? e.payload.text : []))).toEqual([
      'First ',
      'second.',
    ]);
    expect(events.find((e) => e.type === 'core.runState')?.payload).toMatchObject({
      reasons: ['IRL_FAILED'],
    });
    expect(status).toHaveBeenCalledWith('Echo Core reported an output failure.');
    expect(core.failures).toEqual([]);
  } finally {
    bridge.close();
    terminal.stop();
    await core.close();
  }
});
