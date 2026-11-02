import { createRequire } from 'node:module';
import { resolve, join } from 'node:path';
import {
  mkdirSync,
  writeFileSync,
  readFileSync,
  symlinkSync,
  realpathSync,
  existsSync,
} from 'node:fs';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';

const require = createRequire(import.meta.url);
const vite = createRequire(require.resolve('vite'));
const { build } = vite('esbuild');
const out = resolve(process.argv[2] ?? 'D:/Dev/cache/P6-08/cpu-compiled');
mkdirSync(out, { recursive: true });
// Native dependency resolution is recorded explicitly; runtime sample code is bundled unchanged.
const koffi = require.resolve('koffi');
mkdirSync(join(out, 'node_modules'), { recursive: true });
if (!existsSync(join(out, 'node_modules/koffi')))
  symlinkSync(realpathSync(resolve(koffi, '..')), join(out, 'node_modules/koffi'), 'junction');
await build({
  entryPoints: [resolve('packages/desktop-sense/test/cpu-measure.ts')],
  bundle: true,
  platform: 'node',
  target: 'node24',
  format: 'esm',
  external: ['koffi'],
  outfile: join(out, 'measure.mjs'),
});
writeFileSync(join(out, 'package.json'), JSON.stringify({ type: 'module' }));
// createRequire in the compiled production module resolves from this artifact directory.
writeFileSync(join(out, 'native-resolution.txt'), koffi);
const manifest = {
  sourceCommit: execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(),
  file: join(out, 'measure.mjs'),
  sha256: createHash('sha256')
    .update(readFileSync(join(out, 'measure.mjs')))
    .digest('hex'),
  node: process.version,
  nativeDependency: koffi,
};
writeFileSync(join(out, 'build-manifest.json'), JSON.stringify(manifest, null, 2));
console.log(JSON.stringify(manifest));
