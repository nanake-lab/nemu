import { SenseVoiceProvider } from './sensevoice.js';
import { cacheDirectory, verifyModels } from './models.js';
await verifyModels();
const p = new SenseVoiceProvider(cacheDirectory());
const s = p.start({ lang: 'auto', mode: 'continuous' });
s.feed(new Int16Array(320));
s.close();
console.info('Native VAD and SenseVoice loaded successfully');
