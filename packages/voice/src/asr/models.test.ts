import { describe, it, expect } from 'vitest';
import { mkdtemp, writeFile, rm, readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { createHash } from 'node:crypto';
import { modelEntries, verifyModels, downloadModels } from './models.js';
describe('P4-01 frozen models', () => {
  it('keeps asset manifest identical to model lock metadata', async () => {
    const manifest: unknown = JSON.parse(
      await readFile(new URL('./manifest.json', import.meta.url), 'utf8'),
    );
    expect(manifest).toEqual(
      (await modelEntries()).map((f) => ({
        kind: 'model',
        ...f,
        notes: f.redistributable
          ? 'Retain MIT notice'
          : 'SenseVoice redistribution pending user approval',
      })),
    );
  });
  it('rejects modified model bytes and missing/duplicate files', async () => {
    const scratch = await mkdtemp(join(tmpdir(), 'nemu-asr-hash-'));
    try {
      const files = (await modelEntries()).map((f) => ({
        ...f,
        sha256: createHash('sha256').update('fixture').digest('hex'),
      }));
      for (const f of files) await writeFile(join(scratch, f.path), 'fixture');
      const lock = join(scratch, 'lock.json');
      await writeFile(lock, JSON.stringify({ asr: { version: 1, files } }));
      await expect(verifyModels(scratch, lock)).resolves.toBeUndefined();
      await writeFile(join(scratch, 'tokens.txt'), 'corrupt');
      await expect(verifyModels(scratch, lock)).rejects.toThrow('SHA-256 mismatch');
      await writeFile(
        lock,
        JSON.stringify({ asr: { version: 1, files: [files[0], files[0], files[0]] } }),
      );
      await expect(modelEntries(lock)).rejects.toThrow('distinct');
    } finally {
      await rm(scratch, { recursive: true, force: true });
    }
  });
  it('never downloads in CI', async () => {
    const before = process.env.CI;
    process.env.CI = 'true';
    try {
      await expect(downloadModels()).rejects.toThrow('forbidden in CI');
    } finally {
      if (before === undefined) delete process.env.CI;
      else process.env.CI = before;
    }
  });
});
