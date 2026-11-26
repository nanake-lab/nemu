import assert from 'node:assert/strict';
import { writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { performance } from 'node:perf_hooks';
import { AppBus } from '../../../../../apps/desktop/src/bus/bus.js';
import { bindTerminal } from '../../../../../apps/desktop/src/terminal/bridge.js';
import { ChatOutbox, TerminalClient } from '../../../../../packages/terminal/src/index.js';
import { windowsIdentity } from '../../../../../packages/terminal/src/windows.js';
import { EchoCore } from '../../../../../packages/contracts/src/index.js';

const token = process.argv[2]!;
async function until(check: () => boolean) {
  const deadline = performance.now() + 12000;
  while (!check()) {
    assert.ok(performance.now() < deadline, 'desktop did not receive the expected frame');
    await delay(10);
  }
}
function peer(label: string) {
  const bus = new AppBus();
  const statuses: string[] = [];
  const record = (reason: string | null) => {
    if (reason !== null) statuses.push(reason);
  };
  const client = new TerminalClient(new ChatOutbox(join(dirname(token), label)), record, (signal) =>
    windowsIdentity(token, signal),
  );
  const bridge = bindTerminal(bus, client, record);
  client.start(bridge);
  return { bus, client, bridge, statuses };
}
const owner = peer('owner');
const other = peer('other');
try {
  await until(() =>
    [owner, other].every((p) => p.bus.getHistory().some((e) => e.type === 'core.runState')),
  );
  const started = performance.now();
  const first = owner.bus.createEvent('panel', 'chat.send', { text: 'Synthetic first event' });
  owner.bus.publish(first);
  const output = 'test-prefix';
  await until(() =>
    owner.bus.getHistory().some((e) => e.type === 'output.chunk' && e.payload.outputId === output),
  );
  const firstChunkMs = performance.now() - started;
  const cancel = { schema_version: 1, output_id: output };
  assert.equal(
    await other.client.sendControl({
      kind: EchoCore.schema.enums.IngressKind!.INGRESS_CANCEL as EchoCore.IngressKind,
      cancel,
    }),
    false,
  );
  assert.ok(other.statuses.some((s) => s.includes('CAPABILITY_DENIED')));
  await owner.bridge.cancel(cancel);
  const second = owner.bus.createEvent('panel', 'chat.send', { text: 'Synthetic second event' });
  owner.bus.publish(second);
  await until(() =>
    owner.bus
      .getHistory()
      .some((e) => e.type === 'approval.request.v2' && e.payload.approvalId === 'test-approval'),
  );
  const seen = owner.bus.getHistory();
  assert.deepEqual(
    seen.flatMap((e) => (e.type === 'output.chunk' ? e.payload.text : [])),
    ['prefix'],
  );
  assert.ok(
    seen.some(
      (e) =>
        e.type === 'output.state' &&
        e.payload.outputId === output &&
        e.payload.state === 'cancelled',
    ),
  );
  assert.ok(
    seen.some(
      (e) =>
        e.type === 'output.state' &&
        e.payload.outputId === 'test-failure' &&
        e.payload.state === 'cancelled',
    ),
  );
  assert.ok(owner.statuses.includes('Echo Core reported an output failure (test-failure).'));
  assert.ok(
    !other.bus
      .getHistory()
      .some((e) => ['output.chunk', 'output.state', 'approval.request.v2'].includes(e.type)),
  );
  const evidence = process.env.NEMU_P5_53_EVIDENCE_DIR;
  if (evidence) {
    writeFileSync(
      join(evidence, 'two-desktops.json'),
      JSON.stringify(
        {
          firstChunkMs,
          firstInput: first.id,
          secondInput: second.id,
          owner: seen
            .filter((e) => e.type === 'core.runState' || e.type === 'output.state')
            .map((e) => e.payload),
          otherOutputCount: other.bus
            .getHistory()
            .filter((e) => e.type === 'output.chunk' || e.type === 'output.state').length,
          approvalReceived: 'test-approval',
          foreignCancel: 'CAPABILITY_DENIED',
          finalPrefixChunks: 1,
        },
        null,
        2,
      ),
      { flag: 'wx' },
    );
  }
  console.log(
    'Two authenticated desktops, four downlink types, two committed synthetic turns. First event to synthetic chunk ' +
      firstChunkMs.toFixed(1) +
      ' ms.',
  );
} finally {
  for (const p of [owner, other]) {
    p.client.stop();
    p.bridge.close();
  }
}
