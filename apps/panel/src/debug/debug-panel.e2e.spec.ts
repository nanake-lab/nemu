import { _electron as electron, expect, test } from '@playwright/test';
import { resolve, join } from 'node:path';
import { mkdir, mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import type { ElectronApplication } from '@playwright/test';
import type { AppBus } from '../../../desktop/src/bus/index.js';

test('P1-04 debug panel: manual stage.perform command dispatch and event stream recording', async () => {
  const scratch = await mkdtemp(join(tmpdir(), 'nemu-scaffold-debug-panel-'));
  let application: ElectronApplication | undefined;

  try {
    application = await electron.launch({
      args: [
        resolve('apps/desktop'),
        '--use-gl=angle',
        '--use-angle=swiftshader',
        '--enable-unsafe-swiftshader',
      ],
      env: { ...process.env, NEMU_SCAFFOLD_USER_DATA: scratch },
    });

    await expect.poll(() => application?.windows().length).toBe(2);

    const panelPage = application.windows().find((w) => w.url().includes('/panel/'));
    expect(panelPage).toBeDefined();
    if (!panelPage) throw new Error('Panel window missing');

    await panelPage.waitForLoadState('domcontentloaded');

    // 1. Verify developer mode gating (deliverable: 仅在开发模式或设置里开启「开发者模式」后可见)
    const enableDevModeBtn = panelPage.locator('[data-testid="enable-dev-mode-btn"]');
    if (await enableDevModeBtn.isVisible()) {
      await enableDevModeBtn.click();
    }

    // Verify Debug Panel root element is now present and visible
    const debugPanel = panelPage.locator('[data-testid="debug-panel-root"]');
    await expect(debugPanel).toBeVisible({ timeout: 10_000 });

    // 2. Verify CoreStatusViewer displays "未连接" by default
    const coreStatus = panelPage.locator('[data-testid="core-connection-status"]');
    await expect(coreStatus).toHaveText('未连接');

    // 3. Locate Command Sender section
    const commandSelect = panelPage.locator('[data-testid="command-type-select"]');
    await expect(commandSelect).toBeVisible();
    await expect(commandSelect).toHaveValue('stage.perform');

    const validationStatus = panelPage.locator('[data-testid="validation-status"]');
    await expect(validationStatus).toContainText('Schema 校验通过');

    const sendBtn = panelPage.locator('[data-testid="send-command-btn"]');
    await expect(sendBtn).toBeEnabled();

    // 4. Click Send Command to dispatch stage.perform
    await sendBtn.click();

    // Verify transient feedback toast
    const feedbackToast = panelPage.locator('[data-testid="send-feedback-toast"]');
    await expect(feedbackToast).toContainText('成功发送命令: stage.perform');

    // 5. P1-04 核心验收标准:
    // "e2e 测试：发送一条 stage.perform，事件流中出现对应记录。"
    const eventStreamList = panelPage.locator('[data-testid="event-stream-list"]');
    await expect(eventStreamList).toBeVisible();

    const eventRows = eventStreamList.locator('[data-testid="event-row"]');
    await expect(eventRows.first()).toBeVisible({ timeout: 5_000 });

    const firstRowType = eventRows.first().locator('[data-testid="event-type"]');
    await expect(firstRowType).toHaveText('stage.perform');

    const firstRowSource = eventRows.first().locator('[data-testid="event-source"]');
    await expect(firstRowSource).toHaveText('panel');

    const firstRowPreview = eventRows.first().locator('.event-payload-preview');
    await expect(firstRowPreview).toContainText('happy');

    // 6. Test Event Details inspection: click row to view detail pane
    await eventRows.first().click();
    const detailPane = panelPage.locator('[data-testid="event-detail-pane"]');
    await expect(detailPane).toBeVisible();
    await expect(detailPane).toContainText('stage.perform');
    await expect(detailPane).toContainText('"intensity": 0.8');

    // 7. Verify real-time updates from Application Bus via main process
    await application.evaluate(({ app }) => {
      const bus = (app as unknown as { __nemu_bus__?: AppBus }).__nemu_bus__;
      if (!bus) throw new Error('AppBus missing');

      // Publish core.runState
      bus.publish(
        bus.createEvent('main', 'core.runState', {
          states: ['RUNNING', 'OBSERVING'],
          speechMode: 'A_LOCAL_STRICT',
          strict: true,
          lineage: 'REHEARSAL',
          bornAt: '2026-10-20T12:00:00Z',
          autonomy: 'active',
          reasons: ['NOMINAL'],
        }),
      );

      // Publish stage.frameStats
      bus.publish(
        bus.createEvent('stage', 'stage.frameStats', {
          fps: 59.9,
          p95FrameMs: 16.2,
          state: 'idle',
        }),
      );

      // Publish metrics.latency
      bus.publish(
        bus.createEvent('main', 'metrics.latency', {
          utteranceKey: 'utt-e2e-test-1',
          marks: {
            vadEnd: 1000,
            asrDone: 1150,
            received: 1180,
            committed: 1450,
            firstChunk: 1800,
            ttsFirstChunk: 2100,
            audioStart: 2250,
          },
        }),
      );
    });

    // Assert Core Status updated to "已连接" and shows REHEARSAL lineage
    await expect(coreStatus).toHaveText('已连接');
    const lineage = panelPage.locator('[data-testid="core-lineage"]');
    await expect(lineage).toContainText('REHEARSAL');

    // Assert Frame Stats updated to 59.9 fps
    const fpsValue = panelPage.locator('[data-testid="stat-fps-value"]');
    await expect(fpsValue).toHaveText('59.9');

    // Assert Latency waterfall has recorded the utterance
    const statP50 = panelPage.locator('[data-testid="stat-p50"]');
    await expect(statP50).toContainText('1250 ms');

    // 8. Capture screenshots for acceptance report
    const evidenceDir = resolve('reports/P1-04/screenshots');
    await mkdir(evidenceDir, { recursive: true });
    await panelPage.screenshot({ path: join(evidenceDir, 'debug-panel-overview.png') });

    // Switch to event stream tab and screenshot
    const tabEvents = panelPage.locator('[data-testid="tab-events"]');
    await tabEvents.click();
    await panelPage.screenshot({ path: join(evidenceDir, 'debug-panel-events.png') });

    // Switch to latency tab and screenshot
    const tabLatency = panelPage.locator('[data-testid="tab-latency"]');
    await tabLatency.click();
    await panelPage.screenshot({ path: join(evidenceDir, 'debug-panel-latency.png') });
  } finally {
    try {
      await application?.close();
    } finally {
      await rm(scratch, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 });
    }
  }
});
