// P2-05 local review fixture; writes only validated candidate build files.
import { createServer } from 'vite';
import { readFile, writeFile, mkdir, rename } from 'node:fs/promises';
import { resolve } from 'node:path';
import { execFileSync } from 'node:child_process';
import { assetManifestEntrySchema } from '../../../../packages/contracts/dist/index.js';
const root = process.cwd();
const data = JSON.parse(await readFile(resolve(root, 'reports/P2-05/preview-data.json'), 'utf8'));
const candidateRoot = resolve('D:/Dev/nemu/content/_candidates/P2-05');
function category(bones) {
  if (bones[0] === '$tail') return 'tail';
  if (bones.some((n) => /^J_Sec_Hair.*_(23|24)$/.test(n))) return 'ear';
  if (bones.some((n) => n.includes('Hair'))) return 'hair';
  if (bones.some((n) => n.includes('Skirt'))) return 'skirt';
  return null;
}
const fixture = {
  name: 'p205-candidate-review',
  configureServer(server) {
    server.middlewares.use(async (req, res, next) => {
      const files = { '/p205/data': resolve(root, 'reports/P2-05/preview-data.json'), '/p205/nemu.vrm': resolve(data.modelDirectory, 'nemu.vrm'), '/p205/nemu.profile.json': resolve(data.modelDirectory, 'nemu.profile.json') };
      if (files[req.url] && req.method === 'GET') {
        res.setHeader('Content-Type', req.url.endsWith('.vrm') ? 'application/octet-stream' : 'application/json');
        res.end(await readFile(files[req.url])); return;
      }
      if (req.url !== '/p205/save') return next();
      res.setHeader('Content-Type', 'application/json');
      try {
        if (req.method !== 'POST' || req.headers.origin !== 'http://127.0.0.1:5186') throw new Error('Local same-origin POST required');
        const chunks = []; let length = 0;
        for await (const chunk of req) { length += chunk.length; if (length > 65536) throw new Error('Body too large'); chunks.push(chunk); }
        const params = JSON.parse(Buffer.concat(chunks).toString('utf8')).parameters;
        const cfg = structuredClone(data.build);
        for (const chain of cfg.springs.chains) {
          const kind = category(chain.bones); if (!kind) continue;
          for (const key of ['stiffness', 'dragForce', 'gravityPower']) {
            const v = params?.[kind]?.[key];
            if (typeof v !== 'number' || !Number.isFinite(v) || v < 0 || (key === 'dragForce' && v > 1) || (key === 'stiffness' && v > 4) || (key === 'gravityPower' && v > 1)) throw new Error(`Invalid ${kind}.${key}`);
            chain[key] = v;
          }
        }
        const name = `user-tuned-build-${new Date().toISOString().replaceAll(/[:.]/g, '-')}.yaml`;
        const temporary = resolve(root, 'tools/asset-pipeline/.cache', name);
        await mkdir(resolve(root, 'tools/asset-pipeline/.cache'), { recursive: true });
        await writeFile(temporary, JSON.stringify(cfg, null, 2) + '\n');
        const code = 'import sys; from pathlib import Path; sys.path.insert(0,sys.argv[1]); from config import load_config; load_config(Path(sys.argv[2]))';
        execFileSync('D:/Dev/nemu/tools/asset-pipeline/.venv/Scripts/python.exe', ['-c', code, resolve(root, 'tools/asset-pipeline'), temporary], { cwd: root, windowsHide: true, stdio: 'pipe' });
        const target = resolve(candidateRoot, name);
        if (!target.startsWith(candidateRoot + '\\')) throw new Error('Candidate output outside root');
        await rename(temporary, target);
        const manifestPath = resolve(candidateRoot, 'manifest.json');
        const manifest = JSON.parse(await readFile(manifestPath, 'utf8'));
        manifest.push(assetManifestEntrySchema.parse({ path: name, kind: 'other', source: 'P2-05 local review fixture; unapproved tuning of A short-thick candidate', license: 'custom:Nemu private candidate; license pending P7', redistributable: false, author: data.build.meta.authors[0] }));
        await writeFile(manifestPath, JSON.stringify(manifest, null, 2) + '\n');
        res.end(JSON.stringify({ path: target, status: 'candidate-only; not acceptance or formal promotion' }));
      } catch (error) { res.statusCode = 400; res.end(JSON.stringify({ error: String(error) })); }
    });
  },
};
const server = await createServer({ root: resolve(root, 'apps/stage'), configFile: resolve(root, 'apps/stage/vite.config.ts'), plugins: [fixture], server: { host: '127.0.0.1', port: 5186, strictPort: true } });
await server.listen();
console.log('P2-05 review: http://127.0.0.1:5186/test/jitter/review.html');
console.log('Paused by default. Diagnostic playback uses the GPU for 35 seconds; do not overlap a P4 quiet window.');
