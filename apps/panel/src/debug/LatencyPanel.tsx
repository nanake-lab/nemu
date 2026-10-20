import { useState, useMemo } from 'react';
import type { LatencyRecord, LatencyStats } from './types.js';

interface LatencyPanelProps {
  records: LatencyRecord[];
}

export function computeLatencyStats(records: LatencyRecord[]): LatencyStats {
  if (records.length === 0) {
    return { count: 0, p50: 0, p90: 0, min: 0, max: 0, avg: 0 };
  }

  const validDurations = records
    .map((r) => r.totalDurationMs)
    .filter((d): d is number => typeof d === 'number' && !isNaN(d))
    .sort((a, b) => a - b);

  if (validDurations.length === 0) {
    return { count: 0, p50: 0, p90: 0, min: 0, max: 0, avg: 0 };
  }

  const count = validDurations.length;
  const p50Index = Math.floor(count * 0.5);
  const p90Index = Math.min(count - 1, Math.floor(count * 0.9));
  const min = validDurations[0] ?? 0;
  const max = validDurations[count - 1] ?? 0;
  const avg = validDurations.reduce((sum, v) => sum + v, 0) / count;

  return {
    count,
    p50: validDurations[p50Index] ?? 0,
    p90: validDurations[p90Index] ?? 0,
    min,
    max,
    avg: Math.round(avg),
  };
}

export function LatencyPanel({ records }: LatencyPanelProps) {
  const [selectedKey, setSelectedKey] = useState<string | null>(null);

  const stats = useMemo(() => computeLatencyStats(records), [records]);

  const activeRecord = useMemo(() => {
    if (selectedKey) {
      return records.find((r) => r.utteranceKey === selectedKey) ?? records[0] ?? null;
    }
    return records[0] ?? null;
  }, [records, selectedKey]);

  // Waterfall stage calculations for active record
  const waterfallStages = useMemo(() => {
    if (!activeRecord) return [];
    const m = activeRecord.marks;
    const baseTime = m.vadEnd ?? m.received ?? 0;

    const stages: Array<{
      name: string;
      key: string;
      startMs: number;
      endMs: number;
      durationMs: number;
      color: string;
    }> = [];

    const addStage = (
      name: string,
      key: string,
      start?: number,
      end?: number,
      color: string = '#4f46e5',
    ) => {
      if (start !== undefined && end !== undefined && end >= start) {
        stages.push({
          name,
          key,
          startMs: Math.max(0, start - baseTime),
          endMs: Math.max(0, end - baseTime),
          durationMs: Math.max(0, end - start),
          color,
        });
      }
    };

    if (m.vadEnd && m.asrDone) {
      addStage('ASR 识别', 'asr', m.vadEnd, m.asrDone, '#0ea5e9');
    }
    if (m.asrDone && m.received) {
      addStage('排队/传输', 'queue', m.asrDone, m.received, '#6366f1');
    }
    if (m.received && m.committed) {
      addStage('核心提交 (Commit)', 'commit', m.received, m.committed, '#10b981');
    }
    if (m.committed && m.firstChunk) {
      addStage('首块生成 (FirstChunk)', 'firstChunk', m.committed, m.firstChunk, '#f59e0b');
    }
    if (m.firstChunk && m.ttsFirstChunk) {
      addStage('TTS 首包 (TTSChunk)', 'tts', m.firstChunk, m.ttsFirstChunk, '#ec4899');
    }
    if (m.ttsFirstChunk && m.audioStart) {
      addStage('音频启动 (AudioStart)', 'audio', m.ttsFirstChunk, m.audioStart, '#8b5cf6');
    }

    return stages;
  }, [activeRecord]);

  const maxWaterfallMs = useMemo(() => {
    if (waterfallStages.length === 0) return 1000;
    const maxEnd = Math.max(...waterfallStages.map((s) => s.endMs));
    return Math.max(maxEnd, 100);
  }, [waterfallStages]);

  return (
    <div className="debug-section latency-panel-container" data-testid="latency-panel-container">
      <div className="section-header">
        <div className="header-title-row">
          <h3>延迟面板 (Latency & Waterfall)</h3>
          <span className="event-count-badge">最近 {records.length} 次记录</span>
        </div>
        <p className="section-subtitle">
          按 utteranceKey 追踪完整语音往返延迟（VAD → 核心提交 → 首块生成 → 播放）
        </p>
      </div>

      {/* KPI Stats Cards */}
      <div className="latency-stats-grid">
        <div className="latency-stat-card" data-testid="stat-p50">
          <span className="stat-label">p50 延迟</span>
          <span className="stat-number">{stats.p50} ms</span>
          <span className="stat-hint">中位数 (最近50次)</span>
        </div>
        <div className="latency-stat-card" data-testid="stat-p90">
          <span className="stat-label">p90 延迟</span>
          <span className="stat-number">{stats.p90} ms</span>
          <span className="stat-hint">90分位 (最近50次)</span>
        </div>
        <div className="latency-stat-card" data-testid="stat-latest">
          <span className="stat-label">最近单次延迟</span>
          <span className="stat-number">
            {records[0]?.totalDurationMs !== undefined ? `${records[0].totalDurationMs} ms` : '—'}
          </span>
          <span className="stat-hint">{records[0]?.utteranceKey ?? '暂无记录'}</span>
        </div>
        <div className="latency-stat-card" data-testid="stat-range">
          <span className="stat-label">范围 [Min - Max]</span>
          <span className="stat-number">
            {stats.count > 0 ? `${stats.min} ~ ${stats.max} ms` : '—'}
          </span>
          <span className="stat-hint">均值: {stats.avg} ms</span>
        </div>
      </div>

      {/* Waterfall Section */}
      <div className="waterfall-section" data-testid="waterfall-section">
        <div className="waterfall-header">
          <h4>
            瀑布图: {activeRecord ? activeRecord.utteranceKey : '暂无数据'}
            {activeRecord?.totalDurationMs !== undefined && ` (${activeRecord.totalDurationMs} ms)`}
          </h4>
          {records.length > 1 && (
            <div className="utterance-selector">
              <label htmlFor="utterance-select">选择请求:</label>
              <select
                id="utterance-select"
                data-testid="utterance-select"
                value={activeRecord?.utteranceKey ?? ''}
                onChange={(e) => setSelectedKey(e.target.value)}
              >
                {records.map((r) => (
                  <option key={r.utteranceKey} value={r.utteranceKey}>
                    {r.utteranceKey} ({r.totalDurationMs ?? 0} ms)
                  </option>
                ))}
              </select>
            </div>
          )}
        </div>

        {waterfallStages.length === 0 ? (
          <div className="empty-placeholder" data-testid="no-latency-placeholder">
            暂无延迟时间戳数据（等待 metrics.latency 事件）
          </div>
        ) : (
          <div className="waterfall-chart" data-testid="waterfall-chart">
            {waterfallStages.map((stage) => {
              const leftPercent = (stage.startMs / maxWaterfallMs) * 100;
              const widthPercent = Math.max((stage.durationMs / maxWaterfallMs) * 100, 1.5);

              return (
                <div key={stage.key} className="waterfall-row" data-testid={`waterfall-row-${stage.key}`}>
                  <div className="stage-label-col">
                    <span className="stage-name">{stage.name}</span>
                    <span className="stage-duration">+{stage.durationMs}ms</span>
                  </div>
                  <div className="stage-bar-track">
                    <div
                      className="stage-bar"
                      style={{
                        left: `${leftPercent}%`,
                        width: `${widthPercent}%`,
                        backgroundColor: stage.color,
                      }}
                      title={`${stage.name}: ${stage.startMs}ms ~ ${stage.endMs}ms (+${stage.durationMs}ms)`}
                    />
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Raw Marks Table */}
      {activeRecord && (
        <div className="marks-table-wrapper">
          <table className="marks-table">
            <thead>
              <tr>
                <th>阶段标记</th>
                <th>时间戳 (ms)</th>
              </tr>
            </thead>
            <tbody>
              {Object.entries(activeRecord.marks).map(([k, v]) => (
                <tr key={k}>
                  <td><code>{k}</code></td>
                  <td>{v !== undefined ? `${v}` : '—'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
