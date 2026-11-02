import { expect, test } from 'vitest';
import { detectDnd, isMeeting } from './semantics.js';
import { fixtureDesktop, fixtureWindow } from './fixtures.js';
import { idleElapsed } from './win32.js';

for (let i = 0; i < 10; i++) {
  test(`synthetic DND matrix repetition ${i + 1}`, () => {
    const s = fixtureDesktop();
    expect(detectDnd(s, false, false, false)).toBe('none');
    expect(
      detectDnd(
        { ...s, windows: [fixtureWindow({ bounds: s.monitors[0]!.bounds })] },
        false,
        false,
        false,
      ),
    ).toBe('fullscreen');
    expect(
      detectDnd(
        {
          ...s,
          windows: [fixtureWindow({ processName: 'POWERPNT.EXE', windowClass: 'screenClass' })],
        },
        false,
        false,
        false,
      ),
    ).toBe('presentation');
    expect(detectDnd({ ...s, notificationState: 2 }, false, false, false)).toBe('busy');
    expect(detectDnd(s, true, false, false)).toBe('meeting');
    expect(detectDnd(s, true, true, false)).toBe('quietHours');
    expect(
      isMeeting(fixtureWindow({ processName: 'zoom.exe', windowClass: 'ZPContentViewWndClass' })),
    ).toBe(true);
    expect(isMeeting(fixtureWindow({ processName: 'ms-teams.exe' }), 'Synthetic Meeting')).toBe(
      true,
    );
    expect(
      isMeeting(
        fixtureWindow({ processName: 'chrome.exe' }),
        'Meet - abc-defg-hij - Google Chrome',
      ),
    ).toBe(true);
    expect(isMeeting(fixtureWindow({ processName: 'ms-teams.exe' }))).toBe(false);
    expect(isMeeting(fixtureWindow({ processName: 'chrome.exe' }), 'Google Meet help')).toBe(false);
  });
}
test('uncertainty, lock, notification states and presentation priority', () => {
  const s = fixtureDesktop();
  for (const notificationState of [null, 1, 2, 6])
    expect(detectDnd({ ...s, notificationState }, false, false, false)).toBe('busy');
  expect(detectDnd(s, false, false, true)).toBe('busy');
  expect(detectDnd({ ...s, notificationState: 4 }, false, false, false)).toBe('presentation');
  expect(detectDnd({ ...s, notificationState: 3 }, false, false, false)).toBe('fullscreen');
});
test('idle DWORD rollover has exact boundaries', () => {
  expect(idleElapsed(20, 0xfffffff0)).toBe(36);
});
