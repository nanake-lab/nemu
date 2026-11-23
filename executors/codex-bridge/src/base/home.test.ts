import { expect, it } from 'vitest';
import { mkdtemp, mkdir, rmdir, unlink } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { claimHome } from './home.js';

it('owns one isolated CODEX_HOME, rejects a second owner, and releases only its own lock', async () => {
  const root = await mkdtemp(join(tmpdir(), 'nemu-home-test-'));
  const home = join(root, 'isolated'); await mkdir(home);
  try {
    const first = await claimHome(home, root);
    await expect(claimHome(home, root)).rejects.toThrow();
    await first.release(); await first.release();
    const second = await claimHome(home, root); await second.release();
    await expect(claimHome(root, home)).rejects.toThrow('unowned_codex_home');
    await expect(claimHome(root, root)).rejects.toThrow('unowned_codex_home');
  } finally {
    await unlink(join(home, 'nemu-bridge.lock')).catch(() => {});
    await rmdir(home); await rmdir(root);
  }
});
