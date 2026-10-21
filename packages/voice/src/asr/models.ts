import bundledLock from '../../../../config/models.lock.json' with { type: 'json' };
import { createHash } from 'node:crypto';
import { readFile, mkdir, writeFile, rename } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { homedir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { z } from 'zod';
const entry = z.strictObject({
  path: z.enum(['model.int8.onnx', 'tokens.txt', 'silero_vad.onnx']),
  url: z.url(),
  sha256: z.string().regex(/^[a-f0-9]{64}$/),
  revision: z.string(),
  source: z.string(),
  license: z.string(),
  redistributable: z.boolean(),
});
export const cacheDirectory = () =>
  process.env.NEMU_ASR_MODELS ??
  join(process.env.LOCALAPPDATA ?? join(homedir(), '.cache'), 'Nemu', 'models', 'asr');
export async function modelEntries(path?: string) {
  const lock: unknown = path ? JSON.parse(await readFile(path, 'utf8')) : bundledLock;
  return z
    .object({
      asr: z.strictObject({
        version: z.literal(1),
        files: z
          .array(entry)
          .length(3)
          .refine(
            (files) => new Set(files.map((f) => f.path)).size === 3,
            'All three distinct model files are required',
          ),
      }),
    })
    .parse(lock).asr.files;
}
export async function verifyModels(directory = cacheDirectory(), path?: string): Promise<void> {
  for (const f of await modelEntries(path)) {
    const hash = createHash('sha256')
      .update(await readFile(join(directory, f.path)))
      .digest('hex');
    if (hash !== f.sha256) throw new Error(`ASR model SHA-256 mismatch: ${f.path}`);
  }
}
export async function downloadModels(): Promise<void> {
  if (process.env.CI) throw new Error('Model download is local-only; forbidden in CI');
  const directory = cacheDirectory();
  await mkdir(directory, { recursive: true });
  for (const f of await modelEntries()) {
    try {
      const b = await readFile(join(directory, f.path));
      if (createHash('sha256').update(b).digest('hex') === f.sha256) continue;
    } catch {
      /* download missing model */
    }
    const response = await fetch(f.url);
    if (!response.ok) throw new Error(`Model download HTTP ${response.status}: ${f.path}`);
    const b = Buffer.from(await response.arrayBuffer());
    if (createHash('sha256').update(b).digest('hex') !== f.sha256)
      throw new Error(`Downloaded model hash mismatch: ${f.path}`);
    const target = join(directory, f.path);
    await writeFile(`${target}.partial`, b);
    await rename(`${target}.partial`, target);
    console.info(`Verified ${f.path}: ${f.sha256}`);
  }
  await verifyModels(directory);
}
if (
  process.argv[1] &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url) &&
  process.argv[2] === 'download'
)
  await downloadModels();
