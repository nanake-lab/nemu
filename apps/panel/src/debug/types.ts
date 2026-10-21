import type {
  LatencyMarks,
  BodyReadout,
} from '@nemu/contracts';

export type EventSource = 'main' | 'stage' | 'panel' | 'voice';

export interface DebugEventItem {
  id: string;
  ts: number;
  source: EventSource;
  type: string;
  payload: unknown;
  corr?: string | undefined;
  direction?: 'in' | 'out' | undefined;
}

export interface LatencyRecord {
  utteranceKey: string;
  ts: number;
  marks: LatencyMarks;
  totalDurationMs?: number | undefined;
}

export interface LatencyStats {
  count: number;
  p50: number;
  p90: number;
  min: number;
  max: number;
  avg: number;
}

export interface CoreRunStatusState {
  connected: boolean;
  states: string[];
  speechMode?: string | undefined;
  strict?: boolean | undefined;
  lineage?: string | undefined;
  bornAt?: string | undefined;
  autonomy?: 'active' | 'paused' | undefined;
  reasons: string[];
  lastReceipt?: {
    stage: 'RECEIVED_DURABLE' | 'CORE_COMMITTED' | 'FILTERED' | 'REJECTED';
    ts: number;
    eventId?: string | undefined;
    reason?: string | undefined;
  } | undefined;
  recentReceipts: Array<{
    stage: 'RECEIVED_DURABLE' | 'CORE_COMMITTED' | 'FILTERED' | 'REJECTED';
    ts: number;
    eventId?: string | undefined;
    reason?: string | undefined;
  }>;
  lastReadout?: BodyReadout | undefined;
  lastReadoutAt?: number | undefined;
}

export interface FrameStatsState {
  fps: number;
  p95FrameMs: number;
  state: string;
  lastUpdated?: number | undefined;
}
