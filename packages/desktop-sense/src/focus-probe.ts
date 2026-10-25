/** Diagnostic capability only; no cross-process payload or automatic DND policy. */
export interface FocusProbeResult {
  state: 'active' | 'inactive' | 'unknown';
  source: 'wnf' | 'focus-session-manager' | 'unsupported';
  reason: 'ok' | 'unsupported-build' | 'unavailable' | 'query-failed' | 'malformed';
  version: string;
  nativeCode?: number;
  payloadBytes?: number;
  rawValue?: number;
  changeStamp?: number;
}
export interface WnfReply {
  status: number;
  size: number;
  value: number;
  changeStamp: number;
}

/** Only the host build explicitly probed in this task is recognized for undocumented WNF. */
export function readFocusProbe(
  version: string,
  readers: {
    windows10: () => WnfReply;
    windows11: () => Omit<FocusProbeResult, 'version'>;
  },
): FocusProbeResult {
  const match = /^10\.0\.(\d+)(?:\.\d+)?$/.exec(version);
  const build = match ? Number(match[1]) : NaN;
  const source = build >= 22000 ? 'focus-session-manager' : build === 19045 ? 'wnf' : 'unsupported';
  const unknown = (reason: FocusProbeResult['reason']): FocusProbeResult => ({
    state: 'unknown',
    source,
    reason,
    version,
  });
  if (!Number.isSafeInteger(build) || (build >= 22000 && build < 23504) || source === 'unsupported')
    return unknown('unsupported-build');
  try {
    if (source === 'focus-session-manager') return { ...readers.windows11(), version };
    const raw = readers.windows10();
    const metadata = {
      nativeCode: raw.status >>> 0,
      payloadBytes: raw.size,
      rawValue: raw.value,
      changeStamp: raw.changeStamp,
    };
    if (raw.status !== 0) return { ...unknown('query-failed'), ...metadata };
    if (raw.size !== 4 || ![0, 1, 2].includes(raw.value))
      return { ...unknown('malformed'), ...metadata };
    return {
      state: raw.value === 0 ? 'inactive' : 'active',
      source,
      reason: 'ok',
      version,
      ...metadata,
    };
  } catch {
    return unknown('unavailable');
  }
}
