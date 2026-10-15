import { readdir, readFile, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { createHash } from 'node:crypto';

// Apply only to a repository COPY. Raw evidence remains in the CI artifact.
const directory = resolve(process.argv[2]);
const manifestPath = join(directory, 'manifest.json');
const manifest = JSON.parse(await readFile(manifestPath, 'utf8'));
for (const entry of await readdir(directory, { withFileTypes: true })) {
  if (!entry.isFile() || !entry.name.endsWith('.json') || entry.name === 'manifest.json') continue;
  const path = join(directory, entry.name);
  const snapshot = JSON.parse(await readFile(path, 'utf8'));
  if (!Array.isArray(snapshot.processes)) continue;
  snapshot.processes = snapshot.processes.filter((process) =>
    /^(electron|node|nodejs|runner(?:[.-].*)?)$/i.test(process.ProcessName),
  );
  const ids = new Set(snapshot.processes.map((process) => process.Id));
  if (Array.isArray(snapshot.ports)) {
    snapshot.ports = snapshot.ports.filter((port) => ids.has(port.OwningProcess));
  }
  snapshot.processScope =
    'Repository copy: Electron, Node and runner only; ports restricted to these PIDs';
  const bytes = Buffer.from(JSON.stringify(snapshot, null, 2) + '\n');
  await writeFile(path, bytes);
  const record = manifest.files.find((file) => file.path === entry.name);
  if (!record) throw new Error(`Snapshot absent from manifest: ${entry.name}`);
  record.originalArtifactSha256 ??= record.sha256;
  record.sha256 = createHash('sha256').update(bytes).digest('hex').toUpperCase();
  record.bytes = bytes.length;
  record.repositoryFiltered = true;
}
manifest.repositoryCopy = {
  filteredAt: new Date().toISOString(),
  note: 'Filtered derivative. Timestamps and originalArtifactSha256 refer to original capture; sha256 and bytes describe repository files.',
};
await writeFile(manifestPath, JSON.stringify(manifest, null, 2) + '\n');
