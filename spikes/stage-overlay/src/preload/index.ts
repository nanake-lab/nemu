import { contextBridge, ipcRenderer } from 'electron';

export interface SpikeConfig {
  strategy: 'A' | 'B';
  mode: 'interactive' | 'benchmark';
  targetFps: 30 | 60;
  scaleFactor: number;
  durationSeconds: number;
  readPixelsMode: 'async-pbo' | 'sync-readpixels';
  background: 'transparent' | 'white' | 'black' | 'colorful';
  width: number;
  height: number;
  displayBounds: { x: number; y: number; width: number; height: number };
  workArea: { x: number; y: number; width: number; height: number };
  runTest?: string | undefined;
  testName?: string | undefined;
}

export interface MetricSample {
  timestamp: number;
  timeSec: number;
  fps: number;
  frameTimeMs: number;
  cpuPercentTotal: number;
  cpuPercentRenderer: number;
  cpuPercentMain: number;
  cpuPercentGpu: number;
  gpuUtilization: number;
  gpuMemoryUsedMb: number;
  cursorAlpha: number;
  isClickThrough: boolean;
}

const api = {
  getConfig: (): Promise<SpikeConfig> => ipcRenderer.invoke('get-config'),
  setIgnoreMouseEvents: (ignore: boolean, forward = true): void => {
    ipcRenderer.send('set-ignore-mouse-events', ignore, { forward });
  },
  moveWindow: (deltaX: number, deltaY: number): void => {
    ipcRenderer.send('move-window', deltaX, deltaY);
  },
  switchStrategy: (strategy: 'A' | 'B'): Promise<SpikeConfig> => {
    return ipcRenderer.invoke('switch-strategy', strategy);
  },
  captureScreenshot: (filename: string): Promise<string> => {
    return ipcRenderer.invoke('capture-screenshot', filename);
  },
  saveData: (relPath: string, content: string): Promise<boolean> => {
    return ipcRenderer.invoke('save-data', relPath, content);
  },
  recordMetric: (sample: MetricSample): void => {
    ipcRenderer.send('record-metric', sample);
  },
  notifyBenchmarkComplete: (summary: Record<string, unknown>): void => {
    ipcRenderer.send('benchmark-complete', summary);
  },
  onConfigUpdated: (callback: (config: SpikeConfig) => void): (() => void) => {
    const handler = (_event: Electron.IpcRendererEvent, config: SpikeConfig): void =>
      callback(config);
    ipcRenderer.on('config-updated', handler);
    return () => ipcRenderer.removeListener('config-updated', handler);
  },
};

ipcRenderer.on('hardware-sample', (_event, data) => {
  window.postMessage({ type: 'hardware-sample', ...data }, '*');
});

contextBridge.exposeInMainWorld('spikeApi', api);

export type SpikeApi = typeof api;
