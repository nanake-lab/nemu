import { resolve } from 'node:path';
import { lintVrm, type LintResult } from './validator.js';

interface CliOptions {
  file: string;
  profile?: string;
  manifest?: string;
  json: boolean;
}

function usage(): string {
  return 'Usage: pnpm asset:lint <file.vrm> [--profile nemu.profile.json] [--manifest manifest.json] [--json]';
}

function parseArgs(args: string[], workingDirectory: string): CliOptions {
  let file: string | undefined;
  let profile: string | undefined;
  let manifest: string | undefined;
  let json = false;
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === undefined) continue;
    if (arg === '--json') json = true;
    else if (arg === '--profile' || arg === '--manifest') {
      const value = args[index + 1];
      if (!value || value.startsWith('--')) throw new Error(`${arg} requires a path.`);
      if (arg === '--profile') profile = value;
      else manifest = value;
      index += 1;
    } else if (arg.startsWith('--')) throw new Error(`Unknown option: ${arg}`);
    else if (file === undefined) file = arg;
    else throw new Error(`Unexpected argument: ${arg}`);
  }
  if (!file) throw new Error(usage());
  return {
    file: resolve(workingDirectory, file),
    ...(profile ? { profile: resolve(workingDirectory, profile) } : {}),
    ...(manifest ? { manifest: resolve(workingDirectory, manifest) } : {}),
    json,
  };
}

function printText(result: LintResult): void {
  console.log(`${result.file}: ${result.errors} error(s), ${result.warnings} warning(s)`);
  result.diagnostics.forEach((item) => {
    console.log(`${item.severity.toUpperCase()} ${item.rule}: ${item.message}`);
    console.log(`  Fix: ${item.repair}`);
  });
}

try {
  const initialDirectory = resolve(process.env.INIT_CWD ?? process.cwd());
  const options = parseArgs(process.argv.slice(2), initialDirectory);
  const result = await lintVrm(options);
  if (options.json) console.log(JSON.stringify(result, null, 2));
  else printText(result);
  if (result.errors > 0) process.exitCode = 1;
} catch (error) {
  console.error(String(error));
  console.error(usage());
  process.exitCode = 2;
}
