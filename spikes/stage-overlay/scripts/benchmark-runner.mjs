import { spawn } from 'node:child_process';
import { readFileSync, existsSync, readdirSync } from 'node:fs';
import { resolve, join } from 'node:path';

const projectRoot = resolve(import.meta.dirname, '../../..');
const spikeDir = resolve(projectRoot, 'spikes/stage-overlay');
const dataDir = resolve(projectRoot, 'reports/P0-03/data');
const mainEntry = resolve(spikeDir, 'out/main/index.js');

// Parse CLI args
const args = process.argv.slice(2);
function getArg(name, defaultValue) {
  const prefix = `--${name}=`;
  const found = args.find((a) => a.startsWith(prefix));
  return found ? found.substring(prefix.length) : defaultValue;
}
const hasArg = (name) => args.includes(`--${name}`);

const defaultDuration = parseInt(getArg('duration', '300'), 10);
const targetCase = getArg('case', null);

const benchmarkCases = [
  // 100% DPI (2560x1440)
  {
    strategy: 'B',
    fps: 30,
    scale: 1.0,
    duration: defaultDuration,
    name: 'b_idle_100',
    desc: 'Strategy B (Overlay) - Idle 30fps @ 100%',
  },
  {
    strategy: 'B',
    fps: 60,
    scale: 1.0,
    duration: defaultDuration,
    name: 'b_active_100',
    desc: 'Strategy B (Overlay) - Active 60fps @ 100%',
  },
  {
    strategy: 'A',
    fps: 30,
    scale: 1.0,
    duration: defaultDuration,
    name: 'a_idle_100',
    desc: 'Strategy A (Moving Win) - Idle 30fps @ 100%',
  },
  {
    strategy: 'A',
    fps: 60,
    scale: 1.0,
    duration: defaultDuration,
    name: 'a_active_100',
    desc: 'Strategy A (Moving Win) - Active 60fps @ 100%',
  },
  // 150% DPI (2560x1440)
  {
    strategy: 'B',
    fps: 30,
    scale: 1.5,
    duration: defaultDuration,
    name: 'b_idle_150',
    desc: 'Strategy B (Overlay) - Idle 30fps @ 150%',
  },
  {
    strategy: 'B',
    fps: 60,
    scale: 1.5,
    duration: defaultDuration,
    name: 'b_active_150',
    desc: 'Strategy B (Overlay) - Active 60fps @ 150%',
  },
  {
    strategy: 'A',
    fps: 30,
    scale: 1.5,
    duration: defaultDuration,
    name: 'a_idle_150',
    desc: 'Strategy A (Moving Win) - Idle 30fps @ 150%',
  },
  {
    strategy: 'A',
    fps: 60,
    scale: 1.5,
    duration: defaultDuration,
    name: 'a_active_150',
    desc: 'Strategy A (Moving Win) - Active 60fps @ 150%',
  },
];

async function runSingleBenchmark(c) {
  const csvPath = join(dataDir, `${c.name}.csv`);
  if (hasArg('skip-existing') && existsSync(csvPath)) {
    const existingStats = parseCsvStats(csvPath);
    if (existingStats && existingStats.count >= 580) {
      console.log(`Skipping ${c.name}: already has ${existingStats.count} samples (>= 580).`);
      return;
    }
  }

  console.log(`\n======================================================`);
  console.log(`Starting: ${c.desc} [${c.name}] (${c.duration}s)`);
  console.log(`======================================================`);

  const electronArgs = [
    mainEntry,
    `--strategy=${c.strategy}`,
    `--fps=${c.fps}`,
    `--scale=${c.scale}`,
    `--duration=${c.duration}`,
    `--test-name=${c.name}`,
    '--mode=benchmark',
    '--auto-exit',
  ];

  const startTime = Date.now();

  await new Promise((resolvePromise, rejectPromise) => {
    // Run via pnpm exec electron
    const child = spawn('cmd.exe', ['/c', 'pnpm', 'exec', 'electron', ...electronArgs], {
      cwd: projectRoot,
      stdio: 'inherit',
    });

    child.on('error', rejectPromise);
    child.on('exit', (code) => {
      if (code === 0) resolvePromise();
      else rejectPromise(new Error(`Process exited with code ${code}`));
    });
  });

  const elapsed = ((Date.now() - startTime) / 1000).toFixed(1);
  console.log(`Completed ${c.name} in ${elapsed}s.`);

  // Print summary if CSV exists
  if (existsSync(csvPath)) {
    const stats = parseCsvStats(csvPath);
    console.log(`--> Results for ${c.name}:`);
    console.log(`    Samples: ${stats.count}`);
    console.log(
      `    CPU Total: Avg ${stats.avgCpu.toFixed(2)}%, Max ${stats.maxCpu.toFixed(2)}% (Renderer: ${stats.avgRendererCpu.toFixed(2)}%, Main: ${stats.avgMainCpu.toFixed(2)}%, GPU: ${stats.avgGpuCpu.toFixed(2)}%)`,
    );
    console.log(
      `    NVIDIA GPU: Avg ${stats.avgGpu.toFixed(1)}%, Max ${stats.maxGpu.toFixed(1)}%, VRAM ${stats.avgVram.toFixed(0)} MB`,
    );
    console.log(
      `    FPS: Avg ${stats.avgFps.toFixed(1)}, Frame Time: Avg ${stats.avgFrameTime.toFixed(2)}ms, P95 ${stats.p95FrameTime.toFixed(2)}ms`,
    );
  }
}

export function parseCsvStats(filepath) {
  const content = readFileSync(filepath, 'utf-8');
  const lines = content.trim().split(/\r?\n/);
  if (lines.length <= 1) return null;

  const header = lines[0].split(',');
  const getIdx = (col) => header.indexOf(col);

  const idxTime = getIdx('time_sec');
  const idxFps = getIdx('fps');
  const idxFrameTime = getIdx('frame_time_ms');
  const idxCpuTotal = getIdx('cpu_total_pct');
  const idxCpuRend = getIdx('cpu_renderer_pct');
  const idxCpuMain = getIdx('cpu_main_pct');
  const idxCpuGpu = getIdx('cpu_gpu_pct');
  const idxGpuUtil = getIdx('gpu_util_pct');
  const idxGpuMem = getIdx('gpu_mem_mb');

  const cpus = [];
  const rendCpus = [];
  const mainCpus = [];
  const gpuCpus = [];
  const gpus = [];
  const vrams = [];
  const fpss = [];
  const frameTimes = [];

  for (let i = 1; i < lines.length; i++) {
    const p = lines[i].split(',');
    if (p.length < 8) continue;
    cpus.push(parseFloat(p[idxCpuTotal]));
    rendCpus.push(parseFloat(p[idxCpuRend]));
    mainCpus.push(parseFloat(p[idxCpuMain]));
    gpuCpus.push(parseFloat(p[idxCpuGpu]));
    gpus.push(parseFloat(p[idxGpuUtil]));
    vrams.push(parseFloat(p[idxGpuMem]));
    fpss.push(parseFloat(p[idxFps]));
    frameTimes.push(parseFloat(p[idxFrameTime]));
  }

  const avg = (arr) => (arr.length ? arr.reduce((a, b) => a + b, 0) / arr.length : 0);
  const max = (arr) => (arr.length ? Math.max(...arr) : 0);
  const min = (arr) => (arr.length ? Math.min(...arr) : 0);
  const percentile = (arr, p) => {
    if (!arr.length) return 0;
    const sorted = [...arr].sort((a, b) => a - b);
    const index = Math.ceil((p / 100) * sorted.length) - 1;
    return sorted[Math.max(0, Math.min(index, sorted.length - 1))];
  };

  return {
    count: cpus.length,
    avgCpu: avg(cpus),
    maxCpu: max(cpus),
    minCpu: min(cpus),
    avgRendererCpu: avg(rendCpus),
    avgMainCpu: avg(mainCpus),
    avgGpuCpu: avg(gpuCpus),
    avgGpu: avg(gpus),
    maxGpu: max(gpus),
    minGpu: min(gpus),
    avgVram: avg(vrams),
    avgFps: avg(fpss),
    avgFrameTime: avg(frameTimes),
    p95FrameTime: percentile(frameTimes, 95),
    p99FrameTime: percentile(frameTimes, 99),
  };
}

async function main() {
  const filtered = targetCase
    ? benchmarkCases.filter((c) => c.name === targetCase)
    : benchmarkCases;

  console.log(
    `Starting benchmark queue (${filtered.length} runs, duration=${defaultDuration}s each)...`,
  );

  for (const c of filtered) {
    await runSingleBenchmark(c);
  }

  console.log(`\nAll benchmark runs completed!`);
}

if (
  process.argv[1] === new URL(import.meta.url).pathname ||
  process.argv[1].endsWith('benchmark-runner.mjs')
) {
  main().catch((err) => {
    console.error('Benchmark runner error:', err);
    process.exit(1);
  });
}
