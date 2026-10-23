import { homedir } from 'node:os';
import { isAbsolute, join, resolve, win32 } from 'node:path';
/** Windows model/data caches stay off the system drive. Explicit overrides must too. */
function localPath(value: string): string {
  if (!isAbsolute(value)) throw new Error('Nemu cache path must be absolute');
  if (process.platform === 'win32' && win32.parse(resolve(value)).root.toLowerCase() === 'c:\\')
    throw new Error('Nemu model/evaluation caches must not use C:; select a data drive');
  return value;
}
export const nemuCacheRoot = () =>
  localPath(
    process.env.NEMU_CACHE_ROOT ??
      (process.platform === 'win32'
        ? 'D:\\NemuData'
        : join(process.env.XDG_CACHE_HOME ?? join(homedir(), '.cache'), 'Nemu')),
  );
export const asrModelDirectory = () =>
  localPath(process.env.NEMU_ASR_MODELS ?? join(nemuCacheRoot(), 'models', 'asr'));
export const asrDataDirectory = () =>
  localPath(process.env.NEMU_ASR_DATA ?? join(nemuCacheRoot(), 'asr-evaluation'));
export const evaluationDirectory = () => join(nemuCacheRoot(), 'models', 'evaluation');
/** uv-managed Python, package archives, HF/Torch caches and temporary downloads stay together. */
export function evaluationEnvironment(): NodeJS.ProcessEnv {
  const root = nemuCacheRoot();
  return {
    ...process.env,
    NEMU_CACHE_ROOT: root,
    NEMU_ASR_DATA: asrDataDirectory(),
    UV_CACHE_DIR: join(root, 'cache', 'uv'),
    UV_PYTHON_INSTALL_DIR: join(root, 'runtimes', 'python'),
    UV_PYTHON: '3.12',
    UV_PYTHON_PREFERENCE: 'only-managed',
    PYTHONUTF8: '1',
    HF_HOME: join(root, 'cache', 'huggingface'),
    HF_HUB_CACHE: join(root, 'cache', 'huggingface', 'hub'),
    HF_DATASETS_CACHE: join(root, 'cache', 'huggingface', 'datasets'),
    HF_ASSETS_CACHE: join(root, 'cache', 'huggingface', 'assets'),
    TORCH_HOME: join(root, 'cache', 'torch'),
    XDG_CACHE_HOME: join(root, 'cache'),
    TEMP: join(root, 'tmp'),
    TMP: join(root, 'tmp'),
    TMPDIR: join(root, 'tmp'),
  };
}
