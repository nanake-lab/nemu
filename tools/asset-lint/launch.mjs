import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';

const packageDirectory = import.meta.dirname;
const repositoryRoot = resolve(packageDirectory, '../..');
const requestedDirectory = resolve(process.env.INIT_CWD ?? process.cwd());
const compiler = resolve(repositoryRoot, 'node_modules/typescript/bin/tsc');

function compile(project) {
  const result = spawnSync(process.execPath, [compiler, '-p', resolve(repositoryRoot, project)], {
    cwd: repositoryRoot,
    encoding: 'utf8',
  });
  if (result.status !== 0) {
    process.stderr.write(result.stdout ?? '');
    process.stderr.write(result.stderr ?? '');
    process.exit(result.status ?? 1);
  }
}

compile('packages/contracts/tsconfig.json');
compile('tools/asset-lint/tsconfig.json');
const cli = resolve(packageDirectory, 'dist/cli.js');
const result = spawnSync(process.execPath, [cli, ...process.argv.slice(2)], {
  cwd: requestedDirectory,
  stdio: 'inherit',
});
process.exitCode = result.status ?? 1;
