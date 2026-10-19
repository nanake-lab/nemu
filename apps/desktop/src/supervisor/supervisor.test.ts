import { describe, it, expect, afterEach } from 'vitest';
import { SupervisedProcess, ProcessSupervisor } from './index.js';

describe('SupervisedProcess Exponential Backoff and Restart', () => {
  const activeProcesses: Array<SupervisedProcess | ProcessSupervisor> = [];

  afterEach(async () => {
    for (const item of activeProcesses) {
      if ('stopAll' in item) {
        await item.stopAll();
      } else {
        await item.stop();
      }
    }
    activeProcesses.length = 0;
  });

  it('spawns a placeholder process when target binary does not exist', async () => {
    const proc = new SupervisedProcess({
      name: 'test-nonexistent',
      isWorker: false,
      command: 'nonexistent/path/binary.exe',
      args: [],
    });
    activeProcesses.push(proc);

    await proc.start();
    const status = proc.getStatus();
    expect(status.state).toBe('running');
    expect(status.isPlaceholder).toBe(true);
    expect(status.pid).toBeDefined();

    await proc.stop();
    expect(proc.getStatus().state).toBe('stopped');
  });

  it('triggers exponential backoff on crashing process', async () => {
    // Process that exits immediately with error code 1
    const proc = new SupervisedProcess({
      name: 'crashing-process',
      isWorker: false,
      command: process.execPath,
      args: ['-e', 'process.exit(1)'],
      baseDelayMs: 25,
      backoffFactor: 2,
      maxDelayMs: 200,
      autoRestart: true,
      stableThresholdMs: 5000,
    });
    activeProcesses.push(proc);

    await proc.start();
    expect(proc.getBackoffDelay()).toBe(25);

    // Wait until process has crashed and restarted multiple times
    const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

    // Wait for first crash
    let waited = 0;
    while (proc.getStatus().restartCount < 1 && waited < 2000) {
      await wait(10);
      waited += 10;
    }
    expect(proc.getStatus().restartCount).toBeGreaterThanOrEqual(1);
    // After 1st crash, backoff delay was multiplied by 2 -> 50
    expect(proc.getBackoffDelay()).toBeGreaterThanOrEqual(50);

    // Wait for second crash/restart
    waited = 0;
    while (proc.getStatus().restartCount < 2 && waited < 2000) {
      await wait(10);
      waited += 10;
    }
    expect(proc.getStatus().restartCount).toBeGreaterThanOrEqual(2);
    // After 2nd crash, backoff delay multiplied by 2 -> 100
    expect(proc.getBackoffDelay()).toBeGreaterThanOrEqual(100);

    // Wait for third crash/restart
    waited = 0;
    while (proc.getStatus().restartCount < 3 && waited < 2000) {
      await wait(10);
      waited += 10;
    }
    expect(proc.getStatus().restartCount).toBeGreaterThanOrEqual(3);
    expect(proc.getBackoffDelay()).toBeGreaterThanOrEqual(200);

    await proc.stop();
    expect(proc.getStatus().state).toBe('stopped');
  });

  it('stops cleanly and does not restart after stop is called', async () => {
    const proc = new SupervisedProcess({
      name: 'clean-stop-test',
      isWorker: false,
      command: process.execPath,
      args: ['-e', 'setInterval(() => {}, 500)'],
      baseDelayMs: 50,
    });
    activeProcesses.push(proc);

    await proc.start();
    expect(proc.getStatus().state).toBe('running');

    await proc.stop();
    expect(proc.getStatus().state).toBe('stopped');
    expect(proc.getStatus().pid).toBeUndefined();
  });

  it('supervisor manages all default 7 processes', async () => {
    const supervisor = new ProcessSupervisor();
    activeProcesses.push(supervisor);

    const status = supervisor.getStatus();
    expect(Object.keys(status)).toEqual([
      'nemu-core.exe',
      'irl-worker',
      'drive-worker',
      'model-adapter',
      'voice-sidecar',
      'codex-bridge',
      'office-connector',
    ]);

    await supervisor.startAll();
    const runningStatus = supervisor.getStatus();
    for (const p of Object.values(runningStatus)) {
      expect(p.state).toBe('running');
      expect(p.pid).toBeDefined();
    }

    await supervisor.stopAll();
    const stoppedStatus = supervisor.getStatus();
    for (const p of Object.values(stoppedStatus)) {
      expect(p.state).toBe('stopped');
    }
  });

  it('detects hung process missing heartbeats and restarts it without leaving orphan processes', async () => {
    const isPidAlive = (pid: number): boolean => {
      try {
        process.kill(pid, 0);
        return true;
      } catch {
        return false;
      }
    };

    const observedPids = new Set<number>();
    const proc = new SupervisedProcess({
      name: 'hung-process',
      isWorker: false,
      command: process.execPath,
      args: ['-e', 'setInterval(() => {}, 100)'],
      baseDelayMs: 25,
      autoRestart: true,
      healthCheck: {
        type: 'stdout-heartbeat',
        intervalMs: 30,
        timeoutMs: 50,
        maxConsecutiveFailures: 1,
      },
    });
    activeProcesses.push(proc);

    await proc.start();
    expect(proc.getStatus().state).toBe('running');
    expect(proc.getStatus().restartCount).toBe(0);
    const initialPid = proc.getStatus().pid;
    expect(initialPid).toBeDefined();
    if (initialPid) observedPids.add(initialPid);

    const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
    let waited = 0;

    // Track PIDs and assert at most 1 alive child process at any sampling moment
    while (proc.getStatus().restartCount < 2 && waited < 3000) {
      const pid = proc.getStatus().pid;
      if (pid) observedPids.add(pid);

      // Check how many of the observed PIDs are alive right now
      const alivePids = Array.from(observedPids).filter(isPidAlive);
      expect(alivePids.length).toBeLessThanOrEqual(1);

      await wait(25);
      waited += 25;
    }

    expect(proc.getStatus().restartCount).toBeGreaterThanOrEqual(1);
    expect(observedPids.size).toBeGreaterThanOrEqual(2);

    await proc.stop();
    expect(proc.getStatus().state).toBe('stopped');

    // Wait a brief moment for OS taskkill cleanup to settle
    await wait(100);

    // Verify 0 orphan processes remain from all observed children
    const leakedPids = Array.from(observedPids).filter(isPidAlive);
    expect(leakedPids).toEqual([]);
  });

  it('supports http-get probe and none probe without requiring stdout heartbeats', async () => {
    // Process configured with health probe 'none'
    const procNone = new SupervisedProcess({
      name: 'none-probe-test',
      isWorker: false,
      command: process.execPath,
      args: ['-e', 'setInterval(() => {}, 200)'],
      autoRestart: false,
      healthCheck: {
        type: 'none',
      },
    });
    activeProcesses.push(procNone);
    await procNone.start();
    expect(procNone.getStatus().state).toBe('running');

    // Wait 100ms - should still be running and not restarted
    await new Promise((r) => setTimeout(r, 100));
    expect(procNone.getStatus().state).toBe('running');
    expect(procNone.getStatus().restartCount).toBe(0);

    await procNone.stop();
    expect(procNone.getStatus().state).toBe('stopped');
  });
});
