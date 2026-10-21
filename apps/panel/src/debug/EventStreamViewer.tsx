import { useState, useMemo } from 'react';
import type { DebugEventItem, EventSource } from './types.js';

interface EventStreamViewerProps {
  events: DebugEventItem[];
  isPaused: boolean;
  onTogglePause: () => void;
  onClear: () => void;
  pausedBufferCount?: number;
}

export function EventStreamViewer({
  events,
  isPaused,
  onTogglePause,
  onClear,
  pausedBufferCount = 0,
}: EventStreamViewerProps) {
  const [sourceFilter, setSourceFilter] = useState<string>('all');
  const [typeFilter, setTypeFilter] = useState<string>('');
  const [selectedEventId, setSelectedEventId] = useState<string | null>(null);

  const filteredEvents = useMemo(() => {
    return events.filter((ev) => {
      if (sourceFilter !== 'all' && ev.source !== sourceFilter) {
        return false;
      }
      if (typeFilter.trim() !== '') {
        const query = typeFilter.trim().toLowerCase();
        if (!ev.type.toLowerCase().includes(query)) {
          return false;
        }
      }
      return true;
    });
  }, [events, sourceFilter, typeFilter]);

  const selectedEvent = useMemo(() => {
    return events.find((e) => e.id === selectedEventId) ?? null;
  }, [events, selectedEventId]);

  const formatTime = (ts: number): string => {
    const d = new Date(ts);
    const h = String(d.getHours()).padStart(2, '0');
    const m = String(d.getMinutes()).padStart(2, '0');
    const s = String(d.getSeconds()).padStart(2, '0');
    const ms = String(d.getMilliseconds()).padStart(3, '0');
    return `${h}:${m}:${s}.${ms}`;
  };

  const getSourceBadgeClass = (source: EventSource): string => {
    switch (source) {
      case 'main':
        return 'badge-source badge-main';
      case 'stage':
        return 'badge-source badge-stage';
      case 'panel':
        return 'badge-source badge-panel';
      case 'voice':
        return 'badge-source badge-voice';
      default:
        return 'badge-source';
    }
  };

  return (
    <div className="debug-section event-stream-container" data-testid="event-stream-container">
      <div className="section-header">
        <div className="header-title-row">
          <h3>事件流查看器 (Event Stream)</h3>
          <span className="event-count-badge">
            共 {filteredEvents.length} / {events.length} 条
            {isPaused && pausedBufferCount > 0 && ` (${pausedBufferCount} 条未更新)`}
          </span>
        </div>
        <div className="controls-row">
          <div className="filter-group">
            <label htmlFor="source-filter">来源:</label>
            <select
              id="source-filter"
              data-testid="source-filter"
              value={sourceFilter}
              onChange={(e) => setSourceFilter(e.target.value)}
            >
              <option value="all">全部来源</option>
              <option value="main">main</option>
              <option value="stage">stage</option>
              <option value="panel">panel</option>
              <option value="voice">voice</option>
            </select>
          </div>

          <div className="filter-group">
            <label htmlFor="type-filter">类型筛选:</label>
            <input
              id="type-filter"
              type="text"
              data-testid="type-filter"
              placeholder="按事件类型过滤..."
              value={typeFilter}
              onChange={(e) => setTypeFilter(e.target.value)}
            />
          </div>

          <div className="action-buttons">
            <button
              type="button"
              data-testid="pause-btn"
              className={isPaused ? 'btn-warn' : 'btn-secondary'}
              onClick={onTogglePause}
            >
              {isPaused ? '▶ 恢复流' : '⏸ 暂停流'}
            </button>
            <button
              type="button"
              data-testid="clear-btn"
              className="btn-secondary"
              onClick={onClear}
            >
              清空
            </button>
          </div>
        </div>
      </div>

      <div className="event-stream-content">
        <div className="event-list" data-testid="event-stream-list">
          {filteredEvents.length === 0 ? (
            <div className="empty-placeholder">暂无符合条件的事件</div>
          ) : (
            filteredEvents.map((item) => {
              const isSelected = item.id === selectedEventId;
              return (
                <div
                  key={item.id}
                  data-testid="event-row"
                  data-event-id={item.id}
                  className={`event-row ${isSelected ? 'selected' : ''}`}
                  onClick={() => setSelectedEventId(isSelected ? null : item.id)}
                >
                  <span className="event-time">{formatTime(item.ts)}</span>
                  <span className={getSourceBadgeClass(item.source)} data-testid="event-source">
                    {item.source}
                  </span>
                  <span className="event-type" data-testid="event-type">
                    {item.type}
                  </span>
                  <span className="event-payload-preview">
                    {JSON.stringify(item.payload)}
                  </span>
                </div>
              );
            })
          )}
        </div>

        {selectedEvent && (
          <div className="event-detail-pane" data-testid="event-detail-pane">
            <div className="detail-header">
              <h4>事件详情: {selectedEvent.type}</h4>
              <button
                type="button"
                className="btn-close"
                onClick={() => setSelectedEventId(null)}
              >
                ✕
              </button>
            </div>
            <div className="detail-meta">
              <div><strong>ID:</strong> {selectedEvent.id}</div>
              <div><strong>时间:</strong> {new Date(selectedEvent.ts).toISOString()} ({selectedEvent.ts})</div>
              <div><strong>来源:</strong> {selectedEvent.source}</div>
              {selectedEvent.corr && <div><strong>关联 (corr):</strong> {selectedEvent.corr}</div>}
              {selectedEvent.direction && <div><strong>方向:</strong> {selectedEvent.direction}</div>}
            </div>
            <div className="detail-payload">
              <strong>Payload:</strong>
              <pre>{JSON.stringify(selectedEvent.payload, null, 2)}</pre>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
