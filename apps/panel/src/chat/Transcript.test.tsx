import { renderToStaticMarkup } from 'react-dom/server';
import { expect, test } from 'vitest';
import { AppBus } from '../../../desktop/src/bus/bus.js';
import { bindTerminal } from '../../../desktop/src/terminal/bridge.js';
import { scriptedChunk } from '../../../../packages/terminal/test/scripted-core.js';
import { appendChunk, Transcript, type ChatMessage } from './Transcript.js';

test('the transcript renders each actual app-bus chunk and retains its prefix after cancellation', async () => {
  const bus = new AppBus();
  const bridge = bindTerminal(bus, { enqueue() {}, sendControl: async () => true }, () => {});
  let messages: ChatMessage[] = [{ id: 'draft', speaker: 'You', text: 'Synthetic input' }];
  bus.subscribe('output.chunk', (event) => {
    if (event.type === 'output.chunk') messages = appendChunk(messages, event.payload);
  });
  const render = () => renderToStaticMarkup(<Transcript messages={messages} />);
  expect(render()).not.toContain('Nemu');
  bridge.chunk(scriptedChunk('First ', 0n));
  expect(render()).toContain('<strong>Nemu</strong><div>First </div>');
  bridge.chunk(scriptedChunk('second.', 1n));
  expect(render()).toContain('<strong>Nemu</strong><div>First second.</div>');
  await bridge.cancel({ schema_version: 1, output_id: 'scripted-output' });
  bridge.chunk(scriptedChunk(' unseen tail', 2n));
  expect(render()).toContain('<strong>Nemu</strong><div>First second.</div>');
  expect(render()).not.toContain('unseen tail');
  bridge.close();
});
