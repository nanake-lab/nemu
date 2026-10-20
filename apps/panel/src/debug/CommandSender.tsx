import { useState, useId } from 'react';
import {
  STAGE_COMMAND_LIST,
  STAGE_COMMAND_PRESETS,
  type StageCommandType,
  validateJsonAndSchema,
  type ValidationResult,
} from './schema-validator.js';
import type { DebugBusClient } from './bus-client.js';

interface CommandSenderProps {
  busClient: DebugBusClient;
  onCommandSent?: ((type: string, payload: unknown) => void) | undefined;
}

export function CommandSender({ busClient, onCommandSent }: CommandSenderProps) {
  const [selectedType, setSelectedType] = useState<StageCommandType>('stage.perform');
  const [payloadText, setPayloadText] = useState<string>(() => {
    return JSON.stringify(STAGE_COMMAND_PRESETS['stage.perform'], null, 2);
  });
  const [sendFeedback, setSendFeedback] = useState<string | null>(null);

  const commandTypeSelectId = useId();
  const payloadInputId = useId();

  // Validate on the fly
  const validation: ValidationResult = validateJsonAndSchema(selectedType, payloadText);

  const handleTypeChange = (newType: StageCommandType) => {
    setSelectedType(newType);
    const preset = STAGE_COMMAND_PRESETS[newType] ?? {};
    setPayloadText(JSON.stringify(preset, null, 2));
    setSendFeedback(null);
  };

  const handleFormat = () => {
    try {
      const parsed = JSON.parse(payloadText);
      setPayloadText(JSON.stringify(parsed, null, 2));
    } catch {
      // ignore
    }
  };

  const handleSend = () => {
    if (!validation.valid || validation.parsed === undefined) return;

    busClient.sendStageCommand(selectedType, validation.parsed);
    if (onCommandSent) {
      onCommandSent(selectedType, validation.parsed);
    }

    setSendFeedback(`成功发送命令: ${selectedType}`);
    setTimeout(() => {
      setSendFeedback(null);
    }, 4000);
  };

  return (
    <div className="debug-section command-sender-container" data-testid="command-sender-container">
      <div className="section-header">
        <div className="header-title-row">
          <h3>手动发送舞台命令 (Stage Command Dispatcher)</h3>
          {sendFeedback && (
            <span className="toast-success" data-testid="send-feedback-toast">
              {sendFeedback}
            </span>
          )}
        </div>
        <p className="section-subtitle">
          编辑并校验 StageWireMessage 命令，直接向舞台/应用总线发送
        </p>
      </div>

      <div className="command-form">
        <div className="form-row">
          <label htmlFor={commandTypeSelectId}>命令类型 (Type):</label>
          <select
            id={commandTypeSelectId}
            data-testid="command-type-select"
            value={selectedType}
            onChange={(e) => handleTypeChange(e.target.value as StageCommandType)}
          >
            {STAGE_COMMAND_LIST.map((cmd) => (
              <option key={cmd} value={cmd}>
                {cmd}
              </option>
            ))}
          </select>
          <button
            type="button"
            className="btn-secondary btn-sm"
            onClick={handleFormat}
          >
            格式化 JSON
          </button>
        </div>

        <div className="editor-group">
          <label htmlFor={payloadInputId}>载荷 (Payload JSON):</label>
          <textarea
            id={payloadInputId}
            data-testid="command-payload-input"
            className="json-editor-textarea"
            rows={8}
            value={payloadText}
            onChange={(e) => {
              setPayloadText(e.target.value);
              setSendFeedback(null);
            }}
          />
        </div>

        {/* Validation Status Banner */}
        <div className="validation-bar" data-testid="validation-status">
          {validation.valid ? (
            <div className="validation-success">
              ✅ Schema 校验通过 (符合 contracts 规范)
            </div>
          ) : (
            <div className="validation-error">
              <div className="error-title">❌ 校验失败:</div>
              <ul className="error-list">
                {validation.errors?.map((err, idx) => (
                  <li key={idx}>{err}</li>
                ))}
              </ul>
            </div>
          )}
        </div>

        <div className="form-actions">
          <button
            type="button"
            data-testid="send-command-btn"
            className="btn-primary"
            disabled={!validation.valid}
            onClick={handleSend}
          >
            发送命令 ({selectedType})
          </button>
        </div>
      </div>
    </div>
  );
}
