import { setTimeout as delay } from 'node:timers/promises';
/** A late packet invalidates replay rather than sending a burst to catch up. */
export async function replayPcm(
  pcm: Int16Array,
  feed: (block: Int16Array) => void,
  checkQuiet: () => Promise<void>,
  now = () => performance.now(),
  sleep: (ms: number) => Promise<void> = delay,
): Promise<{ startedAt: number; maximumLatenessMs: number }> {
  await checkQuiet();
  let failure: unknown;
  let pending: Promise<void> | undefined;
  const monitor = setInterval(() => {
    if (!pending)
      pending = checkQuiet()
        .catch((e) => {
          failure = e;
        })
        .finally(() => {
          pending = undefined;
        });
  }, 1000);
  const startedAt = now();
  let maximumLatenessMs = 0;
  try {
    for (let offset = 0; offset < pcm.length + 16000; offset += 320) {
      if (failure) throw failure;
      const target = startedAt + (offset + 320) / 16;
      await sleep(Math.max(0, target - now()));
      const lateness = now() - target;
      maximumLatenessMs = Math.max(maximumLatenessMs, lateness);
      if (lateness > 20)
        throw new Error(`Invalid realtime replay: packet late by ${lateness.toFixed(3)} ms`);
      const block = new Int16Array(320);
      block.set(pcm.subarray(offset, offset + 320));
      feed(block);
    }
  } finally {
    clearInterval(monitor);
    await pending;
  }
  if (failure) throw failure;
  return { startedAt, maximumLatenessMs };
}
