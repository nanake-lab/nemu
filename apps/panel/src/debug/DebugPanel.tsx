import { useState, useEffect, useRef, useCallback } from 'react';
import type {
  DebugEventItem,
  CoreRunStatusState,
  FrameStatsState,
  LatencyRecord,
} from './types.js';
import { globalDebugBusClient, type DebugBusClient } from './bus-client.js';
import { EventStreamViewer } from './EventStreamViewer.js';
import { CoreStatusViewer } from './CoreStatusViewer.js';
import { LatencyPanel } from './LatencyPanel.js';
import { FrameStatsViewer } from './FrameStatsViewer.js';
import { CommandSender } from './CommandSender.js';
import './debug.css';

export interface DebugPanelProps {
  busClient?: DebugBusClient | undefined;
  initialDeveloperMode?: boolean | undefined;
}

export function isDeveloperModeActive(): boolean {
  if (typeof window === 'undefined') return true;

  // 1. URL parameter override: ?debug=1 or ?dev=1
  const searchParams = new URLSearchParams(window.location.search);
  if (searchParams.has('debug') || searchParams.has('dev')) {
    return true;
  }

  // 2. Local storage setting
  const stored = localStorage.getItem('nemu:developer_mode');
  if (stored !== null) {
    return stored === 'true';
  }

  // 3. Default to true in non-production builds
  try {
    if (process.env.NODE_ENV !== 'production') {
      return true;
    }
  } catch {
    // ignore
  }

  return false;
}

export function DebugPanel({
  busClient = globalDebugBusClient,
  initialDeveloperMode,
}: DebugPanelProps) {
  const [developerMode, setDeveloperMode] = useState<boolean>(() => {
    return initialDeveloperMode ?? isDeveloperModeActive();
  });

  const [activeTab, setActiveTab] = useState<'all' | 'events' | 'core' | 'latency' | 'frame' | 'command'>('all');

  // Event stream state
  const [events, setEvents] = useState<DebugEventItem[]>([]);
  const [isPaused, setIsPaused] = useState(false);
  const pausedBufferRef = useRef<DebugEventItem[]>([]);
  const [pausedBufferCount, setPausedBufferCount] = useState(0);

  // Core status state - defaults to "未连接" per P1-04 specification before P6-02
  const [coreStatus, setCoreStatus] = useState<CoreRunStatusState>({
    connected: false,
    states: [],
    reasons: [],
    recentReceipts: [],
  });

  // Latency records state (max 50)
  const [latencyRecords, setLatencyRecords] = useState<LatencyRecord[]>([]);

  // Frame stats state
  const [frameStats, setFrameStats] = useState<FrameStatsState | null>(null);

  // Incoming event handler
  const handleEvent = useCallback((item: DebugEventItem) => {
    // 1. Add to event stream
    if (isPaused) {
      pausedBufferRef.current.push(item);
      setPausedBufferCount(pausedBufferRef.current.length);
    } else {
      setEvents((prev) => [item, ...prev].slice(0, 500));
    }

    // 2. Dispatch to specific state viewers
    if (item.type === 'core.runState') {
      const p = item.payload as {
        states?: string[];
        speechMode?: string;
        strict?: boolean;
        lineage?: string;
        bornAt?: string;
        autonomy?: 'active' | 'paused';
        reasons?: string[];
      };
      setCoreStatus((prev) => ({
        ...prev,
        connected: true,
        states: p.states ?? ['RUNNING'],
        speechMode: p.speechMode,
        strict: p.strict,
        lineage: p.lineage,
        bornAt: p.bornAt,
        autonomy: p.autonomy,
        reasons: p.reasons ?? [],
      }));
    } else if (item.type === 'stage.frameStats') {
      const p = item.payload as {
        fps?: number;
        p95FrameMs?: number;
        state?: string;
      };
      setFrameStats({
        fps: p.fps ?? 0,
        p95FrameMs: p.p95FrameMs ?? 0,
        state: p.state ?? 'idle',
        lastUpdated: item.ts,
      });
    } else if (item.type === 'metrics.latency') {
      const p = item.payload as {
        utteranceKey: string;
        marks: {
          vadEnd?: number;
          asrDone?: number;
          received?: number;
          committed?: number;
          firstChunk?: number;
          ttsFirstChunk?: number;
          audioStart?: number;
        };
      };
      if (p?.utteranceKey && p.marks) {
        const base = p.marks.vadEnd ?? p.marks.received ?? 0;
        const end = p.marks.audioStart ?? p.marks.firstChunk ?? p.marks.committed ?? base;
        const totalDurationMs = Math.max(0, end - base);

        setLatencyRecords((prev) => {
          const newRecord: LatencyRecord = {
            utteranceKey: p.utteranceKey,
            ts: item.ts,
            marks: p.marks,
            totalDurationMs,
          };
          return [newRecord, ...prev].slice(0, 50);
        });
      }
    } else if (item.type === 'stage.readout') {
      const p = item.payload as { readout?: CoreRunStatusState['lastReadout'] };
      if (p?.readout) {
        setCoreStatus((prev) => ({
          ...prev,
          lastReadout: p.readout,
          lastReadoutAt: item.ts,
        }));
      }
    } else if (item.type === 'receipt' || item.type.endsWith('.receipt')) {
      const p = item.payload as {
        stage?: 'RECEIVED_DURABLE' | 'CORE_COMMITTED' | 'FILTERED' | 'REJECTED';
        event_id?: string;
        reason?: string;
      };
      if (p?.stage) {
        const receiptEntry = {
          stage: p.stage,
          ts: item.ts,
          eventId: p.event_id,
          reason: p.reason,
        };
        setCoreStatus((prev) => ({
          ...prev,
          lastReceipt: receiptEntry,
          recentReceipts: [receiptEntry, ...prev.recentReceipts.filter((r) => r.stage !== p.stage)].slice(0, 10),
        }));
      }
    }
  }, [isPaused]);

  useEffect(() => {
    const unsub = busClient.subscribe(handleEvent);
    return () => unsub();
  }, [busClient, handleEvent]);

  const handleTogglePause = () => {
    if (isPaused) {
      // Resume: flush paused buffer into events
      setEvents((prev) => [...pausedBufferRef.current.reverse(), ...prev].slice(0, 500));
      pausedBufferRef.current = [];
      setPausedBufferCount(0);
      setIsPaused(false);
    } else {
      setIsPaused(true);
    }
  };

  const handleClearEvents = () => {
    setEvents([]);
    pausedBufferRef.current = [];
    setPausedBufferCount(0);
  };

  const toggleDeveloperMode = () => {
    const next = !developerMode;
    setDeveloperMode(next);
    localStorage.setItem('nemu:developer_mode', String(next));
  };

  // If developer mode is disabled, render only the toggle pill or nothing
  if (!developerMode) {
    return (
      <aside className="dev-mode-floating-toggle" data-testid="dev-mode-disabled-view">
        <button
          type="button"
          className="btn-link"
          data-testid="enable-dev-mode-btn"
          onClick={toggleDeveloperMode}
        >
          ⚙️ 开启开发者模式
        </button>
      </aside>
    );
  }

  return (
    <section className="debug-panel-root" data-testid="debug-panel-root">
      <header className="debug-panel-header">
        <div className="header-brand">
          <h2>🛠 调试面板 (Nemu Debug)</h2>
          <span className="dev-mode-badge" data-testid="dev-mode-badge">
            开发者模式
          </span>
        </div>

        <nav className="tab-navigation" data-testid="debug-tab-navigation">
          <button
            type="button"
            className={`tab-btn ${activeTab === 'all' ? 'active' : ''}`}
            data-testid="tab-all"
            onClick={() => setActiveTab('all')}
          >
            全部概览
          </button>
          <button
            type="button"
            className={`tab-btn ${activeTab === 'events' ? 'active' : ''}`}
            data-testid="tab-events"
            onClick={() => setActiveTab('events')}
          >
            事件流 ({events.length})
          </button>
          <button
            type="button"
            className={`tab-btn ${activeTab === 'core' ? 'active' : ''}`}
            data-testid="tab-core"
            onClick={() => setActiveTab('core')}
          >
            核心状态 ({coreStatus.connected ? '已连接' : '未连接'})
          </button>
          <button
            type="button"
            className={`tab-btn ${activeTab === 'latency' ? 'active' : ''}`}
            data-testid="tab-latency"
            onClick={() => setActiveTab('latency')}
          >
            延迟瀑布 ({latencyRecords.length})
          </button>
          <button
            type="button"
            className={`tab-btn ${activeTab === 'frame' ? 'active' : ''}`}
            data-testid="tab-frame"
            onClick={() => setActiveTab('frame')}
          >
            帧统计 ({frameStats ? `${frameStats.fps.toFixed(0)}fps` : '—'})
          </button>
          <button
            type="button"
            className={`tab-btn ${activeTab === 'command' ? 'active' : ''}`}
            data-testid="tab-command"
            onClick={() => setActiveTab('command')}
          >
            发送命令
          </button>
        </nav>

        <div className="header-actions">
          <button
            type="button"
            className="btn-secondary btn-sm"
            data-testid="disable-dev-mode-btn"
            onClick={toggleDeveloperMode}
          >
            退出开发者模式
          </button>
        </div>
      </header>

      <main className="debug-panel-body">
        {/* All / Overview tab */}
        {activeTab === 'all' && (
          <div className="debug-overview-grid">
            <div className="overview-row">
              <CoreStatusViewer status={coreStatus} />
              <FrameStatsViewer stats={frameStats} />
            </div>
            <LatencyPanel records={latencyRecords} />
            <CommandSender busClient={busClient} />
            <EventStreamViewer
              events={events}
              isPaused={isPaused}
              onTogglePause={handleTogglePause}
              onClear={handleClearEvents}
              pausedBufferCount={pausedBufferCount}
            />
          </div>
        )}

        {/* Individual tabs */}
        {activeTab === 'events' && (
          <EventStreamViewer
            events={events}
            isPaused={isPaused}
            onTogglePause={handleTogglePause}
            onClear={handleClearEvents}
            pausedBufferCount={pausedBufferCount}
          />
        )}

        {activeTab === 'core' && <CoreStatusViewer status={coreStatus} />}

        {activeTab === 'latency' && <LatencyPanel records={latencyRecords} />}

        {activeTab === 'frame' && <FrameStatsViewer stats={frameStats} />}

        {activeTab === 'command' && <CommandSender busClient={busClient} />}
      </main>
    </section>
  );
}
