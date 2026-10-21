import { afterEach, expect, it, vi } from 'vitest';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import {
  asrDataDirectory,
  asrModelDirectory,
  evaluationDirectory,
  evaluationEnvironment,
  nemuCacheRoot,
} from './cache.js';
afterEach(() => vi.unstubAllEnvs());
it('uses D by default on Windows and keeps every evaluation cache under its root', () => {
  vi.stubEnv('NEMU_CACHE_ROOT', undefined);
  vi.stubEnv('NEMU_ASR_MODELS', undefined);
  vi.stubEnv('NEMU_ASR_DATA', undefined);
  const root = nemuCacheRoot();
  if (process.platform === 'win32') expect(root).toBe('D:\\NemuData');
  expect(asrModelDirectory()).toBe(join(root, 'models', 'asr'));
  expect(asrDataDirectory()).toBe(join(root, 'asr-evaluation'));
  expect(evaluationDirectory()).toBe(join(root, 'models', 'evaluation'));
  const env = evaluationEnvironment();
  for (const key of [
    'UV_CACHE_DIR',
    'UV_PYTHON_INSTALL_DIR',
    'HF_HOME',
    'HF_HUB_CACHE',
    'TORCH_HOME',
    'XDG_CACHE_HOME',
    'TEMP',
    'TMP',
  ])
    expect(env[key]?.startsWith(root)).toBe(true);
});
it('honors a non-system-drive root and rejects relative cache paths', () => {
  const root =
    process.platform === 'win32' ? 'D:\\NemuCacheTest' : join(tmpdir(), 'nemu-cache-test');
  vi.stubEnv('NEMU_CACHE_ROOT', root);
  expect(nemuCacheRoot()).toBe(root);
  vi.stubEnv('NEMU_CACHE_ROOT', 'relative-cache');
  expect(nemuCacheRoot).toThrow('absolute');
});
it.skipIf(process.platform !== 'win32')(
  'rejects C drive overrides rather than silently downloading there',
  () => {
    vi.stubEnv('NEMU_CACHE_ROOT', 'C:\\Nemu');
    expect(nemuCacheRoot).toThrow('must not use C:');
    vi.stubEnv('NEMU_CACHE_ROOT', 'D:\\NemuData');
    vi.stubEnv('NEMU_ASR_MODELS', 'C:\\models');
    expect(asrModelDirectory).toThrow('must not use C:');
    vi.stubEnv('NEMU_ASR_DATA', 'C:\\evaluation');
    expect(asrDataDirectory).toThrow('must not use C:');
  },
);
