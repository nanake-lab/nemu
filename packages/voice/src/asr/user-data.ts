import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { z } from 'zod';
import { langSchema } from '@nemu/contracts';
const prompts = z
  .array(z.object({ id: z.string(), lang: langSchema, path: z.string(), text: z.string() }))
  .parse(JSON.parse(await readFile(new URL('./reading-prompts.json', import.meta.url), 'utf8')));
const root =
  process.env.NEMU_ASR_DATA ?? join(process.env.LOCALAPPDATA ?? '', 'Nemu', 'asr-evaluation');
const rows = [];
for (const row of prompts) {
  const dir = join(root, 'user', row.lang);
  await mkdir(dir, { recursive: true });
  const transcript = join(root, row.path.replace(/\.wav$/, '.txt'));
  try {
    await writeFile(transcript, row.text + '\n', { flag: 'wx' });
  } catch (e) {
    if ((e as NodeJS.ErrnoException).code !== 'EEXIST') throw e;
  }
  try {
    const audio = await readFile(join(root, row.path));
    const text = (await readFile(transcript, 'utf8')).trim();
    if (!text) throw new Error(`Empty transcript: ${row.id}`);
    rows.push({ ...row, text, sha256: createHash('sha256').update(audio).digest('hex') });
  } catch (e) {
    if ((e as NodeJS.ErrnoException).code !== 'ENOENT') throw e;
    console.info(`待用户录音: ${row.path}`);
  }
}
await writeFile(join(root, 'user.json'), JSON.stringify(rows, null, 2) + '\n');
console.info(`Registered ${rows.length}/60 user recordings under ${root}`);
