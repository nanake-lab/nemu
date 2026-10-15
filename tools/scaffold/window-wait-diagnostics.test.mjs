import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { EventEmitter } from 'node:events';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';

const source = ts.createSourceFile(
  'smoke.spec.ts',
  readFileSync('apps/desktop/tests/smoke.spec.ts', 'utf8'),
  ts.ScriptTarget.Latest,
  true,
);
let waitBlock;
function visit(node) {
  if (ts.isTryStatement(node) && node.tryBlock.getText(source).includes('await expect.poll(')) {
    // Select the innermost try, whose body is only the original assertion.
    if (node.tryBlock.statements.length === 1) waitBlock = node;
  }
  ts.forEachChild(node, visit);
}
visit(source);
assert.ok(waitBlock?.finallyBlock);
assert.equal(
  waitBlock.tryBlock.statements[0].getText(source),
  'await expect.poll(() => launched.windows().length).toBe(2);',
);
const code = ts.transpileModule(`(async () => { ${waitBlock.getText(source)} })()`, {
  compilerOptions: { target: ts.ScriptTarget.ES2022 },
}).outputText;

for (const fails of [false, true]) {
  test(`snapshot starts after window wait settles (${fails ? 'failure' : 'success'})`, async () => {
    const order = [];
    const originalError = new Error('Original window assertion failure');
    const execution = runInNewContext(code, {
      process: { env: { NEMU_EVIDENCE_DIR: 'fixture' }, platform: 'win32' },
      resolve: (path) => path,
      launched: { windows: () => [] },
      diagnostic: (event) => order.push(event),
      expect: {
        poll: () => ({
          toBe: async () => {
            order.push('wait-start');
            await Promise.resolve();
            order.push('wait-settled');
            if (fails) throw originalError;
          },
        }),
      },
      spawn: () => {
        assert.equal(order.includes('wait-settled'), true);
        order.push('spawn');
        const child = new EventEmitter();
        queueMicrotask(() => child.emit('exit', 0, null));
        return child;
      },
    });
    if (fails) await assert.rejects(execution, (error) => error === originalError);
    else await execution;
    assert.deepEqual(order, [
      'wait-start',
      'wait-settled',
      'window-wait-ended',
      'spawn',
      'snapshot-exit',
    ]);
  });
}

test('repository snapshots contain only related processes and matching derivative hashes', () => {
  for (const directory of ['local-all-checks', 'local-final']) {
    const root = join('reports/P0-01-windows-diagnostics', directory);
    const manifest = JSON.parse(readFileSync(join(root, 'manifest.json'), 'utf8'));
    for (const name of readdirSync(root).filter((name) => name.endsWith('.json'))) {
      const bytes = readFileSync(join(root, name));
      const snapshot = JSON.parse(bytes.toString('utf8'));
      if (!Array.isArray(snapshot.processes)) continue;
      assert.ok(
        snapshot.processes.every((process) =>
          /^(electron|node|nodejs|runner(?:[.-].*)?)$/i.test(process.ProcessName),
        ),
      );
      const ids = new Set(snapshot.processes.map((process) => process.Id));
      assert.ok((snapshot.ports ?? []).every((port) => ids.has(port.OwningProcess)));
      const entry = manifest.files.find((file) => file.path === name);
      assert.equal(entry.repositoryFiltered, true);
      assert.equal(entry.sha256, createHash('sha256').update(bytes).digest('hex').toUpperCase());
    }
  }
});
