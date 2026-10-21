import type { CoreRunStatusState } from './types.js';

interface CoreStatusViewerProps {
  status: CoreRunStatusState;
}

export function CoreStatusViewer({ status }: CoreStatusViewerProps) {
  const isConnected = status.connected;

  const formatTime = (ts?: number): string => {
    if (!ts) return '-';
    const d = new Date(ts);
    return `${d.toLocaleTimeString()}.${String(d.getMilliseconds()).padStart(3, '0')}`;
  };

  const getLineageLabel = (lineage?: string): string => {
    if (!lineage) return '未连接';
    if (lineage === 'REHEARSAL') return 'REHEARSAL (排练谱系)';
    if (lineage === 'LIFE') return 'LIFE (正式谱系)';
    return lineage;
  };

  const getSpeechModeLabel = (mode?: string): string => {
    if (!mode) return '未连接';
    switch (mode) {
      case 'A_LOCAL_STRICT':
        return '模式 A (本地严格)';
      case 'B_REMOTE_PROJECTED':
        return '模式 B (云端投影)';
      case 'C_DRAFT_REWRITE':
        return '模式 C (起草改写)';
      default:
        return mode;
    }
  };

  // Find recent receipts by stage
  const receivedDurable = status.recentReceipts.find(
    (r) => r.stage === 'RECEIVED_DURABLE',
  );
  const coreCommitted = status.recentReceipts.find(
    (r) => r.stage === 'CORE_COMMITTED',
  );

  return (
    <div className="debug-section core-status-container" data-testid="core-status-container">
      <div className="section-header">
        <div className="header-title-row">
          <h3>核心状态查看器 (Echo Core Status)</h3>
          <span
            className={`status-pill ${isConnected ? 'status-connected' : 'status-disconnected'}`}
            data-testid="core-connection-status"
          >
            {isConnected ? '已连接' : '未连接'}
          </span>
        </div>
        <p className="section-subtitle">
          {isConnected
            ? 'Echo Core 实时运行状态与回执'
            : 'P6-02 合并前或核心未就绪时默认显示「未连接」'}
        </p>
      </div>

      <div className="status-grid">
        {/* Run state block */}
        <div className="status-card" data-testid="run-state-card">
          <h4>运行状态</h4>
          <div className="stat-row">
            <span className="stat-label">核心状态:</span>
            <span className="stat-value" data-testid="core-states">
              {isConnected && status.states.length > 0 ? status.states.join(', ') : '未连接'}
            </span>
          </div>
          <div className="stat-row">
            <span className="stat-label">当前谱系:</span>
            <span className="stat-value" data-testid="core-lineage">
              {getLineageLabel(status.lineage)}
            </span>
          </div>
          <div className="stat-row">
            <span className="stat-label">说话模式:</span>
            <span className="stat-value" data-testid="core-speech-mode">
              {getSpeechModeLabel(status.speechMode)}
            </span>
          </div>
          <div className="stat-row">
            <span className="stat-label">严格结论:</span>
            <span className="stat-value" data-testid="core-strict">
              {status.strict !== undefined ? (status.strict ? '是 (strict)' : '否 (relaxed)') : '未连接'}
            </span>
          </div>
          <div className="stat-row">
            <span className="stat-label">自主行动:</span>
            <span className="stat-value" data-testid="core-autonomy">
              {status.autonomy ?? '未连接'}
            </span>
          </div>
          <div className="stat-row">
            <span className="stat-label">诞生时间:</span>
            <span className="stat-value">
              {status.bornAt ?? (isConnected ? '未记录' : '未连接')}
            </span>
          </div>
          {status.reasons.length > 0 && (
            <div className="stat-row">
              <span className="stat-label">状态原因:</span>
              <span className="stat-value">{status.reasons.join(', ')}</span>
            </div>
          )}
        </div>

        {/* Receipts block */}
        <div className="status-card" data-testid="receipts-card">
          <h4>最近回执 (Receipts)</h4>
          <div className="receipt-item">
            <div className="receipt-title">
              <span className="receipt-tag tag-durable">RECEIVED_DURABLE</span>
              <span className="receipt-time">{formatTime(receivedDurable?.ts)}</span>
            </div>
            <div className="receipt-body" data-testid="receipt-durable-info">
              {receivedDurable ? (
                <>
                  <div><strong>Event ID:</strong> {receivedDurable.eventId ?? '—'}</div>
                  {receivedDurable.reason && <div><strong>Reason:</strong> {receivedDurable.reason}</div>}
                </>
              ) : (
                <span className="text-muted">未收到回执</span>
              )}
            </div>
          </div>

          <div className="receipt-item">
            <div className="receipt-title">
              <span className="receipt-tag tag-committed">CORE_COMMITTED</span>
              <span className="receipt-time">{formatTime(coreCommitted?.ts)}</span>
            </div>
            <div className="receipt-body" data-testid="receipt-committed-info">
              {coreCommitted ? (
                <>
                  <div><strong>Event ID:</strong> {coreCommitted.eventId ?? '—'}</div>
                  {coreCommitted.reason && <div><strong>Reason:</strong> {coreCommitted.reason}</div>}
                </>
              ) : (
                <span className="text-muted">未收到回执</span>
              )}
            </div>
          </div>
        </div>

        {/* Body Readout block */}
        <div className="status-card" data-testid="readout-card">
          <h4>最近身体读出 (Body Readout)</h4>
          {status.lastReadout ? (
            <div className="readout-grid">
              <div className="stat-row">
                <span className="stat-label">版本:</span>
                <span className="stat-value">{status.lastReadout.version}</span>
              </div>
              <div className="stat-row">
                <span className="stat-label">节奏系数 (idleTempo):</span>
                <span className="stat-value">{status.lastReadout.idleTempo.toFixed(2)}</span>
              </div>
              <div className="stat-row">
                <span className="stat-label">猫耳压角 (earPose):</span>
                <span className="stat-value">{status.lastReadout.earPose.toFixed(2)}</span>
              </div>
              <div className="stat-row">
                <span className="stat-label">尾巴摆幅 (tailAmplitude):</span>
                <span className="stat-value">{status.lastReadout.tailAmplitude.toFixed(2)}</span>
              </div>
              <div className="stat-row">
                <span className="stat-label">社交倾向 (gazeSociability):</span>
                <span className="stat-value">{status.lastReadout.gazeSociability.toFixed(2)}</span>
              </div>
              <div className="stat-row">
                <span className="stat-label">姿态偏置 (posture):</span>
                <span className="stat-value">{status.lastReadout.posture.toFixed(2)}</span>
              </div>
              <div className="stat-row">
                <span className="stat-label">表情基线:</span>
                <span className="stat-value">
                  {JSON.stringify(status.lastReadout.expressionBaseline)}
                </span>
              </div>
            </div>
          ) : (
            <div className="text-muted" data-testid="no-readout-placeholder">
              {isConnected ? '暂无身体读出记录' : '未连接 / 暂无身体读出'}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
