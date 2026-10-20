import type { FrameStatsState } from './types.js';

interface FrameStatsViewerProps {
  stats: FrameStatsState | null;
}

export function FrameStatsViewer({ stats }: FrameStatsViewerProps) {
  const fps = stats?.fps ?? 0;
  const p95 = stats?.p95FrameMs ?? 0;
  const state = stats?.state ?? '未连接';

  const getFpsColor = (v: number): string => {
    if (v >= 55) return '#10b981'; // Green
    if (v >= 28) return '#f59e0b'; // Amber
    return '#ef4444'; // Red
  };

  const getP95Color = (ms: number): string => {
    if (ms === 0) return '#6b7280';
    if (ms <= 18) return '#10b981'; // Good (<= 60fps budget)
    if (ms <= 36) return '#f59e0b'; // Acceptable (<= 30fps budget)
    return '#ef4444'; // High frame drops
  };

  const formatTime = (ts?: number): string => {
    if (!ts) return '未收到数据';
    const d = new Date(ts);
    return `${d.toLocaleTimeString()}.${String(d.getMilliseconds()).padStart(3, '0')}`;
  };

  return (
    <div className="debug-section frame-stats-container" data-testid="frame-stats-container">
      <div className="section-header">
        <div className="header-title-row">
          <h3>帧统计 (Stage Frame Stats)</h3>
          <span className="event-count-badge" data-testid="frame-stats-updated">
            {formatTime(stats?.lastUpdated)}
          </span>
        </div>
        <p className="section-subtitle">
          来自舞台渲染进程的实时性能遥测（每 5 秒上报一次）
        </p>
      </div>

      <div className="frame-stats-grid">
        <div className="stat-card" data-testid="stat-card-fps">
          <span className="stat-label">渲染帧率 (FPS)</span>
          <span
            className="stat-big-number"
            style={{ color: getFpsColor(fps) }}
            data-testid="stat-fps-value"
          >
            {stats ? fps.toFixed(1) : '—'}
          </span>
          <span className="stat-hint">
            {stats ? (fps >= 50 ? '流畅运行 (60fps)' : fps >= 25 ? '待机节流 (30fps)' : '掉帧/降级') : '等待舞台上报'}
          </span>
        </div>

        <div className="stat-card" data-testid="stat-card-p95">
          <span className="stat-label">p95 帧时间</span>
          <span
            className="stat-big-number"
            style={{ color: getP95Color(p95) }}
            data-testid="stat-p95-value"
          >
            {stats ? `${p95.toFixed(1)} ms` : '—'}
          </span>
          <span className="stat-hint">
            {stats ? '95% 帧在此时间内完成渲染' : '等待舞台上报'}
          </span>
        </div>

        <div className="stat-card" data-testid="stat-card-state">
          <span className="stat-label">当前物理状态 (PhysicalState)</span>
          <span className="stat-state-badge" data-testid="stat-physical-state">
            {state}
          </span>
          <span className="stat-hint">
            {stats ? `当前角色身体动作模式: ${state}` : '未连接 / 舞台未加载'}
          </span>
        </div>
      </div>
    </div>
  );
}
