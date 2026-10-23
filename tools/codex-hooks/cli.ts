import { parseArgs } from 'node:util';
import { install, uninstall } from './install.js';

try {
  const { values, positionals } = parseArgs({
    allowPositionals: true,
    options: { 'config-dir': { type: 'string' }, 'state-dir': { type: 'string' } },
  });
  const [action] = positionals;
  if (
    positionals.length !== 1 ||
    !['install', 'uninstall'].includes(action ?? '') ||
    !values['config-dir'] ||
    !values['state-dir']
  ) {
    throw new Error(
      'Usage: install|uninstall --config-dir <absolute path> --state-dir <absolute path>',
    );
  }
  const options = { configDir: values['config-dir'], stateDir: values['state-dir'] };
  await (action === 'install' ? install(options) : uninstall(options));
  process.stdout.write(`Codex session hooks ${action === 'install' ? 'enabled' : 'disabled'}.\n`);
} catch {
  process.stderr.write(
    'Codex session hook configuration failed. Check explicit paths and valid hooks.json; no automatic fallback.\n',
  );
  process.exitCode = 1;
}
