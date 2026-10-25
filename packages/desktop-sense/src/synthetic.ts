import { fixtureDesktop } from './fixtures.js';
import type { DesktopSource } from './types.js';

/** Scaffold and test mode never enumerate or inspect the actual desktop. */
export function createSyntheticSource(): DesktopSource {
  let disposed = false;
  return {
    sample() {
      if (disposed) throw new Error('Source disposed');
      return fixtureDesktop();
    },
    readTitle() {
      return undefined;
    },
    subscribe() {
      return () => {};
    },
    dispose() {
      disposed = true;
    },
  };
}
