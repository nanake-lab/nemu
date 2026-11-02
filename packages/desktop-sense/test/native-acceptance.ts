import assert from 'node:assert/strict';
import { writeFileSync } from 'node:fs';
import { createSyntheticWin32Fixture } from '../src/win32.js';
import { buildPlatforms } from '../src/geometry.js';

const output = process.argv[2];
if (!output) throw new Error('Provide a new evidence JSON path');
const fixture = createSyntheticWin32Fixture(8);
try {
  const platforms = buildPlatforms(fixture.source.sample());
  const cases = fixture.frames.map(({ hwnd, expected, scale }) => {
    const actual = platforms.find((p) => p.hwnd === hwnd);
    assert.ok(actual);
    const errorPx = Math.max(
      Math.abs(actual.x1 - expected.x),
      Math.abs(actual.x2 - expected.x - expected.w),
      Math.abs(actual.y - expected.y),
    );
    return { scale, expected, actual, errorPx };
  });
  const passed = cases.every((c) => c.errorPx <= 2);
  writeFileSync(
    output,
    JSON.stringify(
      {
        status: passed ? 'passed' : 'failed',
        scope: 'native synthetic windows; synthetic display bounds; does not change OS scaling',
        cases,
      },
      null,
      2,
    ),
    { flag: 'wx' },
  );
  assert.ok(passed);
  console.log(JSON.stringify({ status: 'passed', cases }));
} finally {
  fixture.dispose();
}
