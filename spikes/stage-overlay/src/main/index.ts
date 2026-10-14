import { app, BrowserWindow, ipcMain, screen } from 'electron';
import { join, resolve } from 'node:path';
import { writeFileSync, mkdirSync } from 'node:fs';
import { spawn, type ChildProcess } from 'node:child_process';
import type { SpikeConfig, MetricSample } from '../preload/index';

// Command line switches
const args = process.argv.slice(2);
function getArg(name: string, defaultValue: string): string {
  const prefix = `--${name}=`;
  const found = args.find((a) => a.startsWith(prefix));
  return found ? found.substring(prefix.length) : defaultValue;
}
const hasArg = (name: string): boolean => args.includes(`--${name}`);

const initialStrategy = (getArg('strategy', 'B').toUpperCase() === 'A' ? 'A' : 'B') as 'A' | 'B';
const initialMode = (
  getArg('mode', 'interactive') === 'benchmark' || hasArg('bench-all') ? 'benchmark' : 'interactive'
) as 'interactive' | 'benchmark';
const initialFps = (parseInt(getArg('fps', '30'), 10) === 60 ? 60 : 30) as 30 | 60;
const scaleArg = parseFloat(getArg('scale', '1.0'));
const initialScale = scaleArg > 1.2 ? 1.5 : 1.0;
const initialDuration = parseInt(getArg('duration', '300'), 10);
const initialReadPixels = (
  getArg('readpixels', 'async-pbo') === 'sync-readpixels' ? 'sync-readpixels' : 'async-pbo'
) as 'async-pbo' | 'sync-readpixels';
const initialBg = getArg('bg', 'transparent') as 'transparent' | 'white' | 'black' | 'colorful';
const runTestArg = getArg('run-test', '');
const testNameArg = getArg('test-name', '');

// If scale is specified, append switch before ready
if (initialScale > 1.0) {
  app.commandLine.appendSwitch('force-device-scale-factor', initialScale.toString());
}
app.commandLine.appendSwitch('enable-gpu-rasterization');
app.commandLine.appendSwitch('ignore-gpu-blocklist');

let mainWindow: BrowserWindow | null = null;
let currentConfig: SpikeConfig;
let latestGpuUtil = 0;
let latestGpuMemMb = 0;
let nvidiaSmiProc: ChildProcess | null = null;
const recordedMetrics: MetricSample[] = [];

function startGpuMonitoring(): void {
  try {
    nvidiaSmiProc = spawn('nvidia-smi', [
      '--query-gpu=timestamp,utilization.gpu,memory.used',
      '--format=csv,noheader,nounits',
      '-lms',
      '500',
    ]);

    let buffer = '';
    nvidiaSmiProc.stdout?.on('data', (chunk: Buffer) => {
      buffer += chunk.toString();
      const lines = buffer.split(/\r?\n/);
      buffer = lines.pop() ?? '';
      for (const line of lines) {
        const parts = line.split(',').map((s) => s.trim());
        if (parts.length >= 3) {
          const util = parseFloat(parts[1] ?? '0');
          const mem = parseFloat(parts[2] ?? '0');
          if (!isNaN(util)) latestGpuUtil = util;
          if (!isNaN(mem)) latestGpuMemMb = mem;
        }
      }
    });

    nvidiaSmiProc.on('error', (err) => {
      console.warn('Failed to start nvidia-smi monitoring:', err.message);
    });
  } catch (err) {
    console.warn('nvidia-smi not available:', err);
  }
}

function stopGpuMonitoring(): void {
  if (nvidiaSmiProc && !nvidiaSmiProc.killed) {
    try {
      nvidiaSmiProc.kill();
    } catch (err) {
      void err;
    }
  }
}

function sampleProcessMetrics(): {
  totalCpu: number;
  rendererCpu: number;
  mainCpu: number;
  gpuCpu: number;
} {
  const appMetrics = app.getAppMetrics();
  let rendererCpu = 0;
  let mainCpu = 0;
  let gpuCpu = 0;

  for (const m of appMetrics) {
    if (m.type === 'Tab') {
      rendererCpu += m.cpu.percentCPUUsage;
    } else if (m.type === 'Browser') {
      mainCpu += m.cpu.percentCPUUsage;
    } else if (m.type === 'GPU') {
      gpuCpu += m.cpu.percentCPUUsage;
    }
  }

  const totalCpu = rendererCpu + mainCpu + gpuCpu;
  return { totalCpu, rendererCpu, mainCpu, gpuCpu };
}

function getWorkAreaAndDisplay(): {
  workArea: { x: number; y: number; width: number; height: number };
  bounds: { x: number; y: number; width: number; height: number };
} {
  const primary = screen.getPrimaryDisplay();
  return {
    workArea: {
      x: primary.workArea.x,
      y: primary.workArea.y,
      width: primary.workArea.width,
      height: primary.workArea.height,
    },
    bounds: {
      x: primary.bounds.x,
      y: primary.bounds.y,
      width: primary.bounds.width,
      height: primary.bounds.height,
    },
  };
}

function createMainWindow(strategy: 'A' | 'B'): BrowserWindow {
  const { workArea, bounds } = getWorkAreaAndDisplay();

  const charWidth = 600;
  const charHeight = 800;

  let winX: number;
  let winY: number;
  let winW: number;
  let winH: number;

  if (strategy === 'A') {
    winW = charWidth;
    winH = charHeight;
    winX = workArea.x + workArea.width - charWidth - 60;
    winY = workArea.y + workArea.height - charHeight - 60;
  } else {
    // Strategy B: Full primary work area
    winX = workArea.x;
    winY = workArea.y;
    winW = workArea.width;
    winH = workArea.height;
  }

  currentConfig = {
    strategy,
    mode: initialMode,
    targetFps: initialFps,
    scaleFactor: initialScale,
    durationSeconds: initialDuration,
    readPixelsMode: initialReadPixels,
    background: initialBg,
    width: winW,
    height: winH,
    displayBounds: bounds,
    workArea,
    runTest: runTestArg || undefined,
    testName: testNameArg || undefined,
  };

  const win = new BrowserWindow({
    x: winX,
    y: winY,
    width: winW,
    height: winH,
    transparent: true,
    frame: false,
    alwaysOnTop: true,
    skipTaskbar: true,
    hasShadow: false,
    resizable: false,
    backgroundColor: '#00000000',
    webPreferences: {
      preload: join(import.meta.dirname, '../preload/index.cjs'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false,
    },
  });

  win.setAlwaysOnTop(true, 'screen-saver');

  win.webContents.on('console-message', (_event, _level, message) => {
    console.log(`[Renderer] ${message}`);
  });

  // Load renderer
  const devUrl = process.env.ELECTRON_RENDERER_URL;
  if (devUrl) {
    win.loadURL(devUrl);
  } else {
    win.loadFile(join(import.meta.dirname, '../renderer/index.html'));
  }

  return win;
}

app.whenReady().then(() => {
  startGpuMonitoring();

  mainWindow = createMainWindow(initialStrategy);

  // Periodically send hardware metrics to renderer every 500ms
  setInterval(() => {
    if (!mainWindow || mainWindow.isDestroyed()) return;
    const cpu = sampleProcessMetrics();
    mainWindow.webContents.send('hardware-sample', {
      timestamp: Date.now(),
      cpu,
      gpu: {
        utilization: latestGpuUtil,
        memoryUsedMb: latestGpuMemMb,
      },
    });
  }, 500);

  // IPC handlers
  ipcMain.handle('get-config', () => currentConfig);

  let lastIgnoreState: boolean | null = null;
  ipcMain.on(
    'set-ignore-mouse-events',
    (_event, ignore: boolean, options?: { forward: boolean }) => {
      if (mainWindow && !mainWindow.isDestroyed()) {
        if (lastIgnoreState !== ignore) {
          lastIgnoreState = ignore;
          mainWindow.setIgnoreMouseEvents(ignore, options ?? { forward: true });
        }
      }
    },
  );

  ipcMain.on('move-window', (_event, deltaX: number, deltaY: number) => {
    if (mainWindow && !mainWindow.isDestroyed() && currentConfig.strategy === 'A') {
      const [currX = 0, currY = 0] = mainWindow.getPosition();
      mainWindow.setPosition(currX + deltaX, currY + deltaY);
    }
  });

  ipcMain.handle('switch-strategy', (_event, newStrategy: 'A' | 'B') => {
    if (!mainWindow || mainWindow.isDestroyed()) return currentConfig;
    const { workArea } = getWorkAreaAndDisplay();
    const charWidth = 600;
    const charHeight = 800;

    if (newStrategy === 'A') {
      const x = workArea.x + workArea.width - charWidth - 60;
      const y = workArea.y + workArea.height - charHeight - 60;
      mainWindow.setBounds({ x, y, width: charWidth, height: charHeight });
    } else {
      mainWindow.setBounds({
        x: workArea.x,
        y: workArea.y,
        width: workArea.width,
        height: workArea.height,
      });
    }

    currentConfig.strategy = newStrategy;
    mainWindow.webContents.send('config-updated', currentConfig);
    return currentConfig;
  });

  ipcMain.handle('capture-screenshot', async (_event, filename: string) => {
    if (!mainWindow || mainWindow.isDestroyed()) return '';
    const image = await mainWindow.webContents.capturePage();
    const outDir = resolve(process.cwd(), 'reports/P0-03/screenshots');
    mkdirSync(outDir, { recursive: true });
    const targetPath = join(outDir, filename);
    writeFileSync(targetPath, image.toPNG());
    console.log(`[Main] Screenshot saved to ${targetPath}`);
    return targetPath;
  });

  ipcMain.handle('save-data', (_event, relPath: string, content: string) => {
    try {
      const fullPath = resolve(process.cwd(), relPath);
      const dir = resolve(fullPath, '..');
      mkdirSync(dir, { recursive: true });
      writeFileSync(fullPath, content, 'utf-8');
      console.log(`[Main] Data saved to ${fullPath}`);
      return true;
    } catch (err) {
      console.error('[Main] Failed to save data:', err);
      return false;
    }
  });

  ipcMain.on('record-metric', (_event, sample: MetricSample) => {
    recordedMetrics.push(sample);
  });

  ipcMain.on('benchmark-complete', (_event, summary: Record<string, unknown>) => {
    console.log('[Main] Benchmark completed:', summary);
    if (initialMode === 'benchmark' || hasArg('auto-exit')) {
      setTimeout(() => {
        app.quit();
      }, 1000);
    }
  });
});

app.on('window-all-closed', () => {
  stopGpuMonitoring();
  app.quit();
});

app.on('will-quit', () => {
  stopGpuMonitoring();
});
