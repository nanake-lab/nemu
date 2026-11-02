import { powerMonitor } from 'electron';
import { createWin32Source, createSyntheticSource, type DesktopSource } from '@nemu/desktop-sense';
import { startDesktopSense } from './index.js';

/** Shell composition kept in one place for subsequent P6-11 wiring. */
export function startShellSense(
  options: Omit<Parameters<typeof startDesktopSense>[0], 'source' | 'session'> & {
    source?: DesktopSource;
  },
): () => void {
  const source =
    options.source ??
    (process.env.NEMU_SCAFFOLD_USER_DATA
      ? createSyntheticSource()
      : createWin32Source({ titleAuthorized: options.settings.titleAuthorized ?? (() => false) }));
  return startDesktopSense({ ...options, source, session: powerMonitor });
}
