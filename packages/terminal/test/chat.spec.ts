import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { EchoCore } from '@nemu/contracts';
import { expect, test } from '@playwright/test';
import { AppBus } from '../../../apps/desktop/src/bus/bus.js';
import { bindTerminal } from '../../../apps/desktop/src/terminal/bridge.js';
import type { ChatStatus } from '../../../apps/panel/src/chat/channel.js';
import { ChatOutbox, TerminalClient } from '../src/index.js';
import { scriptedChunk, scriptedCore } from './scripted-core.js';

declare global {
  interface Window {
    fakeCoreBus?: MessagePort;
    fakeCoreStatus?: (status: ChatStatus) => void;
    toAppBus(message: unknown): Promise<void>;
    cancelFromPanel(message: unknown): Promise<void>;
  }
}

test('chat marks the rejected row, streams received chunks, and keeps the visible prefix after Stop', async ({
  page,
}) => {
  const dir = await mkdtemp(join(tmpdir(), 'nemu-p6-renderer-'));
  const core = await scriptedCore(dir, {
    rejectFirstEvent: EchoCore.schema.enums.IngressReason!.OBJECT_MISMATCH as NonNullable<
      EchoCore.TerminalResponse['reason']
    >,
  });
  const bus = new AppBus();
  let status: ChatStatus = { canSend: true, reason: 'Connecting to synthetic core…' };
  const report = (reason: string | null, disconnected = false, rejectedEventId?: string) => {
    let next: ChatStatus;
    if (reason !== null && rejectedEventId !== undefined) {
      next = { ...status, rejection: { eventId: rejectedEventId, reason } };
    } else {
      status = { canSend: true, reason, disconnected };
      next = status;
    }
    void page.evaluate((value) => window.fakeCoreStatus?.(value), next).catch(() => {});
  };
  const terminal = new TerminalClient(new ChatOutbox(join(dir, 'outbox')), report, core.identity);
  const bridge = bindTerminal(bus, terminal, report);
  let toMain: ((message: { data: unknown }) => void) | undefined;
  bus.connectPort('panel', {
    on: (_type, receive) => {
      toMain = receive;
    },
    postMessage: (data) => {
      void page.evaluate((message) => window.fakeCoreBus?.postMessage(message), data);
    },
  });

  try {
    await page.exposeBinding('toAppBus', (_source, data: unknown) => toMain?.({ data }));
    await page.exposeBinding('cancelFromPanel', (_source, data: unknown) => bridge.cancel(data));
    await page.addInitScript((initialStatus: ChatStatus) => {
      window.nemuChat = {
        onStatus(listener) {
          window.fakeCoreStatus = listener;
          listener(initialStatus);
          return () => {
            delete window.fakeCoreStatus;
          };
        },
        cancel: (message) => {
          void window.cancelFromPanel(message);
        },
      };
      // Use the same IPC-delivered MessagePort shape the real panel receives.
      window.addEventListener('load', () => {
        const { port1, port2 } = new MessageChannel();
        window.fakeCoreBus = port1;
        port1.onmessage = (event) => {
          void window.toAppBus(event.data);
        };
        window.postMessage({ type: 'nemu:channel:bus' }, '*', [port2]);
      });
    }, status);
    await page.goto('/');
    terminal.start();
    const region = page.getByRole('region', { name: 'Conversation' });
    const list = region.getByRole('list', { name: 'Messages' });
    const composer = region.getByRole('textbox', { name: 'Message', exact: true });
    await composer.fill('Synthetic rejected input');
    await region.getByRole('button', { name: 'Send' }).click();
    const rejectedRow = list.getByRole('listitem').filter({ hasText: 'Synthetic rejected input' });
    const rejection = rejectedRow.getByText('没送到：Core rejected the request (OBJECT_MISMATCH)');
    await expect(rejection).toBeVisible();
    await expect(rejection).toHaveCSS('color', 'rgb(89, 99, 115)');

    await composer.fill('Synthetic conversation input');
    await region.getByRole('button', { name: 'Send' }).click();
    await expect.poll(() => [...core.messages.values()]).toEqual(['Synthetic conversation input']);
    await expect(list.getByRole('listitem')).toHaveCount(2);
    await expect(list.getByRole('listitem').last()).not.toContainText('没送到');
    await expect(rejection).toBeVisible();

    bridge.chunk(scriptedChunk('First ', 0n));
    await expect(list.getByRole('listitem').last()).toContainText('First');
    await expect(list.getByRole('listitem').last()).not.toContainText('second');
    bridge.chunk(scriptedChunk('second.', 1n));
    await expect(list.getByRole('listitem').last()).toContainText('First second.');

    await region.getByRole('button', { name: 'Stop' }).click();
    await expect.poll(() => core.controls[0]?.cancel?.output_id).toBe('scripted-output');
    bridge.chunk(scriptedChunk(' late tail', 2n));
    bridge.end({ schema_version: 1, output_id: 'scripted-output', reason: 2 });
    await expect(region.getByRole('button', { name: 'Stop' })).toHaveCount(0);
    await expect(list.getByRole('listitem').last()).toHaveText('NemuFirst second.');
    bridge.end({ schema_version: 1, output_id: 'failed-turn', reason: 3 });
    await expect(region.getByRole('status')).toHaveText('Echo Core reported an output failure.');
    expect(core.failures).toEqual([]);
    if (process.env.NEMU_P6_EVIDENCE) {
      await page.screenshot({ path: join(process.env.NEMU_P6_EVIDENCE, 'panel.png') });
    }
  } finally {
    bridge.close();
    terminal.stop();
    await core.close();
    await rm(dir, { recursive: true, force: true });
  }
});
