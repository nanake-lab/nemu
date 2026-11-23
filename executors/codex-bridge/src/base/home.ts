import { open, realpath, unlink } from 'node:fs/promises';
import { homedir } from 'node:os';
import { isAbsolute, join, relative, resolve } from 'node:path';

/** Only claim an already-provisioned, dedicated home beneath the application-owned root. */
export async function claimHome(codeHome: string, ownedRoot: string): Promise<{ home: string; release: () => Promise<void> }> {
  const [home, root] = await Promise.all([realpath(codeHome), realpath(ownedRoot)]);
  const rel = relative(root, home);
  if (!rel || rel === '..' || rel.startsWith('..\\') || rel.startsWith('../') || isAbsolute(rel)) throw new Error('unowned_codex_home');
  if (process.env.CODEX_HOME && resolve(process.env.CODEX_HOME).toLowerCase() === home.toLowerCase()) throw new Error('inherited_codex_home');
  if (resolve(join(homedir(), '.codex')).toLowerCase() === home.toLowerCase()) throw new Error('user_codex_home');
  const path = join(home, 'nemu-bridge.lock');
  const handle = await open(path, 'wx');
  let releasing: Promise<void> | null = null;
  return { home, release: async () => {
    releasing ??= (async () => { await handle.close(); await unlink(path); })();
    await releasing;
  } };
}
