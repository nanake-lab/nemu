import { createVoiceRuntime } from '../../../../packages/voice/src/asr/runtime.js';
const parent = process.parentPort;
if (!parent) throw new Error('Voice entry requires Electron utilityProcess');
const runtime = await createVoiceRuntime(
  (event) => parent.postMessage(event),
  process.env.NEMU_ASR_LOCK,
);
parent.on('message', (event) => {
  if (event.ports.length === 2) {
    const [mic, audio] = event.ports;
    if (mic && audio) runtime.attach(mic, audio);
  } else runtime.command(event.data);
});
process.on('exit', () => runtime.close());
parent.postMessage('ready');
