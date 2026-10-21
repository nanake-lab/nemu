import { describe, it, expect } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { DebugPanel } from './DebugPanel.js';
import { CoreStatusViewer } from './CoreStatusViewer.js';
import { FrameStatsViewer } from './FrameStatsViewer.js';
import { LatencyPanel } from './LatencyPanel.js';
import { CommandSender } from './CommandSender.js';
import { EventStreamViewer } from './EventStreamViewer.js';
import { DebugBusClient } from './bus-client.js';
import type { CoreRunStatusState, FrameStatsState, LatencyRecord, DebugEventItem } from './types.js';

describe('DebugPanel and sub-components markup rendering', () => {
  it('renders DebugPanel root when developer mode is enabled', () => {
    const html = renderToStaticMarkup(
      <DebugPanel initialDeveloperMode={true} />,
    );
    expect(html).toContain('调试面板 (Nemu Debug)');
    expect(html).toContain('开发者模式');
    expect(html).toContain('全部概览');
    expect(html).toContain('事件流');
    expect(html).toContain('核心状态');
    expect(html).toContain('延迟瀑布');
    expect(html).toContain('帧统计');
    expect(html).toContain('发送命令');
  });

  it('renders disabled view when developer mode is false', () => {
    const html = renderToStaticMarkup(
      <DebugPanel initialDeveloperMode={false} />,
    );
    expect(html).toContain('开启开发者模式');
    expect(html).not.toContain('全部概览');
  });

  it('renders CoreStatusViewer showing "未连接" by default before P6-02 is connected', () => {
    const disconnectedState: CoreRunStatusState = {
      connected: false,
      states: [],
      reasons: [],
      recentReceipts: [],
    };
    const html = renderToStaticMarkup(<CoreStatusViewer status={disconnectedState} />);
    expect(html).toContain('未连接');
    expect(html).toContain('P6-02 合并前或核心未就绪时默认显示「未连接」');
    expect(html).toContain('未收到回执');
    expect(html).toContain('未连接 / 暂无身体读出');
  });

  it('renders CoreStatusViewer with active run state and receipts when connected', () => {
    const connectedState: CoreRunStatusState = {
      connected: true,
      states: ['RUNNING', 'OBSERVING'],
      speechMode: 'A_LOCAL_STRICT',
      strict: true,
      lineage: 'REHEARSAL',
      bornAt: '2026-10-20T12:00:00Z',
      autonomy: 'active',
      reasons: ['NOMINAL'],
      lastReceipt: {
        stage: 'CORE_COMMITTED',
        ts: Date.now(),
        eventId: '01AN4Z07BY79KA1307SR9X4MV3',
      },
      recentReceipts: [
        {
          stage: 'RECEIVED_DURABLE',
          ts: Date.now() - 50,
          eventId: '01AN4Z07BY79KA1307SR9X4MV2',
        },
        {
          stage: 'CORE_COMMITTED',
          ts: Date.now(),
          eventId: '01AN4Z07BY79KA1307SR9X4MV3',
        },
      ],
      lastReadout: {
        version: 'v1',
        expressionBaseline: { happy: 0.1 },
        idleTempo: 1.05,
        earPose: -0.2,
        tailAmplitude: 0.6,
        gazeSociability: 0.9,
        posture: 0.1,
      },
    };

    const html = renderToStaticMarkup(<CoreStatusViewer status={connectedState} />);
    expect(html).toContain('已连接');
    expect(html).toContain('RUNNING, OBSERVING');
    expect(html).toContain('REHEARSAL (排练谱系)');
    expect(html).toContain('模式 A (本地严格)');
    expect(html).toContain('是 (strict)');
    expect(html).toContain('01AN4Z07BY79KA1307SR9X4MV2');
    expect(html).toContain('01AN4Z07BY79KA1307SR9X4MV3');
    expect(html).toContain('1.05');
  });

  it('renders FrameStatsViewer with live fps, p95 and physical state', () => {
    const stats: FrameStatsState = {
      fps: 59.8,
      p95FrameMs: 16.4,
      state: 'speaking',
      lastUpdated: Date.now(),
    };
    const html = renderToStaticMarkup(<FrameStatsViewer stats={stats} />);
    expect(html).toContain('59.8');
    expect(html).toContain('16.4 ms');
    expect(html).toContain('speaking');
  });

  it('renders LatencyPanel with waterfall stages and statistics', () => {
    const records: LatencyRecord[] = [
      {
        utteranceKey: 'utt-demo-1',
        ts: Date.now(),
        marks: {
          vadEnd: 1000,
          asrDone: 1200,
          received: 1220,
          committed: 1500,
          firstChunk: 1900,
          ttsFirstChunk: 2200,
          audioStart: 2350,
        },
        totalDurationMs: 1350,
      },
    ];

    const html = renderToStaticMarkup(<LatencyPanel records={records} />);
    expect(html).toContain('utt-demo-1');
    expect(html).toContain('1350 ms');
    expect(html).toContain('ASR 识别');
    expect(html).toContain('核心提交 (Commit)');
    expect(html).toContain('首块生成 (FirstChunk)');
    expect(html).toContain('TTS 首包 (TTSChunk)');
    expect(html).toContain('音频启动 (AudioStart)');
  });

  it('renders CommandSender with stage.perform selected by default and valid schema', () => {
    const client = new DebugBusClient();
    const html = renderToStaticMarkup(<CommandSender busClient={client} />);
    expect(html).toContain('stage.perform');
    expect(html).toContain('Schema 校验通过');
    expect(html).toContain('发送命令 (stage.perform)');
    client.destroy();
  });

  it('renders EventStreamViewer with events and controls', () => {
    const events: DebugEventItem[] = [
      {
        id: '01TEST00000000000000000001',
        ts: Date.now(),
        source: 'panel',
        type: 'stage.perform',
        payload: { cue: { kind: 'emo', value: 'happy' } },
        direction: 'out',
      },
    ];

    const html = renderToStaticMarkup(
      <EventStreamViewer
        events={events}
        isPaused={false}
        onTogglePause={() => {}}
        onClear={() => {}}
      />,
    );
    expect(html).toContain('01TEST00000000000000000001');
    expect(html).toContain('stage.perform');
    expect(html).toContain('panel');
    expect(html).toContain('暂停流');
    expect(html).toContain('清空');
  });
});
