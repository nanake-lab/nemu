import { expect, test } from 'vitest';
import { buildPlatforms } from './geometry.js';
import { fixtureDesktop, fixtureWindow } from './fixtures.js';

for (const scale of [1, 1.25, 1.5, 2])
  test(`physical edges at ${scale * 100}% on two synthetic monitors`, () => {
    const r = { x: -1600 + 80 * scale, y: -300 + 40 * scale, w: 600 * scale, h: 400 * scale };
    const sample = fixtureDesktop({
      windows: [fixtureWindow({ bounds: r })],
      monitors: [
        {
          id: 'left',
          bounds: { x: -1600, y: -300, w: 1600, h: 1200 },
          workArea: { x: -1600, y: -300, w: 1600, h: 1160 },
          dpiX: 96 * scale,
          dpiY: 96 * scale,
        },
        ...fixtureDesktop().monitors,
      ],
    });
    const platform = buildPlatforms(sample).find((p) => p.hwnd === '100')!;
    expect(platform).toMatchObject({ x1: r.x, x2: r.x + r.w, y: r.y });
    expect(buildPlatforms(sample).filter((p) => p.id.startsWith('screen:'))).toHaveLength(2);
  });

test('only exposed spans are standable; no minimized, invisible, cloaked or off-screen windows', () => {
  const sample = fixtureDesktop({
    windows: [
      fixtureWindow({ hwnd: 'front', bounds: { x: 300, y: 50, w: 100, h: 300 } }),
      fixtureWindow({ hwnd: 'back', z: 1 }),
      fixtureWindow({ hwnd: 'min', minimized: true }),
      fixtureWindow({ hwnd: 'hidden', visible: false }),
      fixtureWindow({ hwnd: 'cloak', cloaked: true }),
      fixtureWindow({ hwnd: 'off', bounds: { x: -3000, y: 1, w: 20, h: 20 } }),
    ],
  });
  expect(
    buildPlatforms(sample)
      .filter((p) => p.hwnd === 'back')
      .map((p) => [p.x1, p.x2, p.y]),
  ).toEqual([
    [100, 300, 100],
    [400, 700, 100],
  ]);
  expect(
    buildPlatforms(sample).some((p) => ['min', 'hidden', 'cloak', 'off'].includes(p.hwnd)),
  ).toBe(false);
});

test('clips spanning windows per monitor and drops an offscreen top', () => {
  const sample = fixtureDesktop({
    windows: [fixtureWindow({ bounds: { x: -100, y: 30, w: 300, h: 400 } })],
  });
  expect(buildPlatforms(sample)[0]).toMatchObject({ x1: 0, x2: 200, y: 30 });
  sample.windows[0]!.bounds.y = -10;
  expect(buildPlatforms(sample).some((p) => p.hwnd === '100')).toBe(false);
});
test('platform identity survives window motion and screen edges remain behind sparse z ranks', () => {
  const sample = fixtureDesktop({ windows: [fixtureWindow({ z: 100 })] });
  const first = buildPlatforms(sample).find((p) => p.hwnd === '100')!;
  sample.windows[0]!.bounds.x += 10;
  expect(buildPlatforms(sample).find((p) => p.hwnd === '100')!.id).toBe(first.id);
  expect(buildPlatforms(sample).find((p) => p.id.startsWith('screen:'))!.z).toBe(101);
});
