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

const { IngressReason } = EchoCore.schema.enums;
type RejectionReason = NonNullable<EchoCore.TerminalResponse['reason']>;

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
  reopened.remove(event.id);
  expect(readdirSync(path)).toEqual([]);
  expect([...new ChatOutbox(path).pending()]).toEqual([]);
});

test('a permanently rejected message is not resent after the terminal and outbox restart', async () => {
  const path = temp();
  const outboxPath = join(path, 'outbox');
  const core = await scriptedCore(path, {
    rejectFirstEvent: IngressReason!.OBJECT_MISMATCH as RejectionReason,
  });
  const status = vi.fn();
  const terminal = new TerminalClient(new ChatOutbox(outboxPath), status, core.identity);
  const bus = new AppBus();
  const rejected = bus.createEvent('panel', 'chat.send', {
    text: 'Synthetic rejected',
  }) as ChatEvent;
  const next = bus.createEvent('panel', 'chat.send', {
    text: 'Synthetic after restart',
  }) as ChatEvent;
  let restarted: TerminalClient | undefined;
  try {
    terminal.enqueue(rejected);
    terminal.start();
    await vi.waitFor(() =>
      expect(status).toHaveBeenCalledWith(
        expect.stringContaining('OBJECT_MISMATCH'),
        false,
        rejected.id,
      ),
    );
    terminal.stop();

    const reopened = new ChatOutbox(outboxPath);
    expect([...reopened.pending()]).toEqual([]);
    restarted = new TerminalClient(reopened, status, core.identity);
    restarted.enqueue(next);
    restarted.start();
    await vi.waitFor(() => expect(core.messages.get(next.id)).toBe(next.payload.text));
    expect(core.input.map((event) => event.stable_source_key)).toEqual([rejected.id, next.id]);
    expect(core.failures).toEqual([]);
  } finally {
    terminal.stop();
    restarted?.stop();
    await core.close();
  }
});

test('retryable ingress backpressure reconnects and resends the same observation', async () => {
  const path = temp();
  const outboxPath = join(path, 'outbox');
  const core = await scriptedCore(path, {
    rejectFirstEvent: IngressReason!.PENDING_RECORDS_FULL as RejectionReason,
  });
  const status = vi.fn();
  const terminal = new TerminalClient(new ChatOutbox(outboxPath), status, core.identity);
  const event = new AppBus().createEvent('panel', 'chat.send', {
    text: 'Synthetic retry',
  }) as ChatEvent;
  try {
    terminal.enqueue(event);
    terminal.start();
    await vi.waitFor(() => expect(readdirSync(outboxPath)).toEqual([]), { timeout: 4000 });
    expect(status).toHaveBeenCalledWith(expect.stringContaining('PENDING_RECORDS_FULL'), true);
    expect(core.timestamps.authenticated).toHaveLength(2);
    expect(core.input).toHaveLength(2);
    expect(core.input[0]?.stable_source_key).toBe(event.id);
    expect(core.input[0]).toEqual(core.input[1]);
    expect(core.messages.get(event.id)).toBe(event.payload.text);
    expect(core.failures).toEqual([]);
  } finally {
    terminal.stop();
    await core.close();
  }
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

test('a delayed rejection belongs to the original durable event after a different event settles', async () => {
  const path = temp();
  let held: EchoCore.TerminalResponse | undefined;
  const core = await scriptedCore(path, {
    settle: (current) => {
      if (held) return [current, held];
      held = {
        ...current,
        receipt: {
          ...current.receipt!,
          stage: 4,
          reason_code: IngressReason!.SOURCE_KEY_CONFLICT as RejectionReason,
        },
      };
      return [];
    },
  });
  const status = vi.fn();
  const outbox = new ChatOutbox(join(path, 'outbox'));
  const terminal = new TerminalClient(outbox, status, core.identity);
  const bus = new AppBus();
  try {
    terminal.start();
    const first = bus.createEvent('panel', 'chat.send', { text: 'First' }) as ChatEvent;
    const second = bus.createEvent('panel', 'chat.send', { text: 'Second' }) as ChatEvent;
    terminal.enqueue(first);
    await vi.waitFor(() => expect([...outbox.pending()]).toEqual([]));
    expect(status).not.toHaveBeenCalledWith(expect.any(String), false, first.id);
    terminal.enqueue(second);
    await vi.waitFor(() =>
      expect(status).toHaveBeenCalledWith(
        expect.stringContaining('SOURCE_KEY_CONFLICT'),
        false,
        first.id,
      ),
    );
    expect(status).not.toHaveBeenCalledWith(expect.any(String), false, second.id);
    expect([...outbox.pending()]).toEqual([]);
    expect(core.timestamps.authenticated).toHaveLength(1);
    expect(core.failures).toEqual([]);
  } finally {
    terminal.stop();
    await core.close();
  }
});

test('authenticated mixed frames reach the bridge between the durable and settlement receipts', async () => {
  const path = temp();
  const core = await scriptedCore(path, {
    framesBeforeSettlement: [
      encodeFrame('OutputChunk', scriptedChunk('First ', 0n)),
      encodeFrame('OutputChunk', scriptedChunk('second.', 1n)),
      encodeFrame('OutputChunk', scriptedChunk('duplicate', 1n)),
    ],
  });
  const bus = new AppBus();
  const status = vi.fn<(reason: string | null) => void>();
  const terminal = new TerminalClient(new ChatOutbox(join(path, 'outbox')), status, core.identity);
  const bridge = bindTerminal(bus, terminal, status);
  try {
    terminal.start(bridge);
    const event = bus.createEvent('panel', 'chat.send', { text: 'Synthetic turn' });
    bus.publish(event);
    await vi.waitFor(() =>
      expect(bus.getHistory().filter((e) => e.type === 'output.chunk')).toHaveLength(2),
    );
    expect(readdirSync(join(path, 'outbox'))).toEqual([]);
    await bridge.cancel({ schema_version: 1, output_id: 'scripted-output' });
    core.push(
      encodeFrame('OutputChunk', scriptedChunk('late tail', 2n)),
      encodeFrame('OutputEnd', { schema_version: 1, output_id: 'scripted-output', reason: 2 }),
      encodeFrame('OutputEnd', { schema_version: 1, output_id: 'failed-turn', reason: 3 }),
      encodeFrame('RunStateV2', {
        schema_version: 1,
        states: [6],
        speech_mode: 2,
        strict: false,
        autonomy: 2,
        reasons: ['IRL_FAILED'],
        lineage: 1,
      }),
      encodeFrame('ApprovalRequestV2', scriptApproval),
    );
    await vi.waitFor(() =>
      expect(bus.getHistory().some((e) => e.type === 'approval.request.v2')).toBe(true),
    );
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
    expect(status).toHaveBeenCalledWith('Echo Core reported an output failure (failed-turn).');
    expect(core.failures).toEqual([]);
  } finally {
    bridge.close();
    terminal.stop();
    await core.close();
  }
});
