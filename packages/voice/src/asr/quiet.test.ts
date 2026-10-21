import { describe, it, expect } from 'vitest';
import { busyProcesses } from './quiet.js';
const p = (Name: string, ProcessId: number, CommandLine: string | null, ParentProcessId = 1) => ({
  Name,
  ProcessId,
  CommandLine,
  ParentProcessId,
});
describe('Quiet host measurement', () => {
  it('allows idle runner and ordinary control servers but rejects a running CI worker', () => {
    expect(
      busyProcesses(
        [p('Runner.Listener.exe', 2, 'listener'), p('node.exe', 3, 'node server.mjs')],
        undefined,
        99,
      ),
    ).toEqual([]);
    expect(busyProcesses([p('Runner.Worker.exe', 4, 'job')], undefined, 99)).toHaveLength(1);
  });
  it('detects Python GPU probes and node Playwright tests without exposing command lines', () => {
    const found = busyProcesses(
      [
        p('python.exe', 5, 'python D:\\Dev\\nemu-p007\\workers\\model-adapter\\probe\\run_all.py'),
        p('node.exe', 6, 'node playwright cli test'),
      ],
      undefined,
      99,
    );
    expect(found.map((r) => r.ProcessId)).toEqual([5, 6]);
    expect(JSON.stringify(found)).not.toContain('run_all');
  });
  it('excludes the current measuring process and only the explicit native Electron fixture', () => {
    const rows = [
      p('node.exe', 77, 'tsx measure.ts'),
      p('electron.exe', 90, 'native fixture'),
      p('electron.exe', 91, 'utility', 90),
      p('electron.exe', 92, 'other test'),
    ];
    expect(busyProcesses(rows, 90, 77).map((r) => r.ProcessId)).toEqual([92]);
  });
  it('blocks an uninspectable worker and native compilation', () => {
    expect(
      busyProcesses([p('python.exe', 7, null), p('rustc.exe', 8, 'compile')], undefined, 99).map(
        (r) => r.reason,
      ),
    ).toEqual(['cannot-inspect-worker', 'heavy-process']);
  });
});
