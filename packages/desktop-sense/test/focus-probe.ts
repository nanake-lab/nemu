import { writeFileSync } from 'node:fs';
import { probeFocusState } from '../src/win32.js';
const output = process.argv[2];
if (!output) throw new Error('Provide a fresh output path');
const result = {
  at: new Date().toISOString(),
  scope: 'read-only feasibility; no settings or window-content access',
  result: probeFocusState(),
};
writeFileSync(output, JSON.stringify(result, null, 2), { flag: 'wx' });
console.log(JSON.stringify(result));
