import {
  SenseEngine,
  startSenseLoop,
  type DesktopSource,
  type SenseSettings,
  type SenseSink,
} from '@nemu/desktop-sense';
import type { AppBus } from '../bus/bus.js';
import type { StagePayloads } from '@nemu/contracts';

export interface SessionSignals {
  on(event: 'lock-screen' | 'unlock-screen', listener: () => void): unknown;
  removeListener(event: 'lock-screen' | 'unlock-screen', listener: () => void): unknown;
}

/** Inject existing main-process ports. Geometry never enters AppBus or the core. */
export function startDesktopSense(options: {
  source: DesktopSource;
  settings: SenseSettings;
  bus: Pick<AppBus, 'createEvent' | 'publish' | 'subscribe'>;
  sendStage: (type: 'stage.desktop.update', payload: StagePayloads['stage.desktop.update']) => void;
  session: SessionSignals;
  onFault: () => void;
}): () => void {
  let latest: StagePayloads['stage.desktop.update'] | undefined;
  const sink: SenseSink = {
    body: (payload) => {
      latest = payload;
      options.sendStage('stage.desktop.update', payload);
    },
    bus: (type, payload) => {
      options.bus.publish(options.bus.createEvent('main', type, payload));
    },
  };
  const engine = new SenseEngine(options.source, options.settings, sink);
  const lock = () => engine.setLocked(true);
  const unlock = () => engine.setLocked(false);
  const stopLoop = startSenseLoop(options.source, engine, options.onFault);
  const stopReady = options.bus.subscribe('stage.ready', () => {
    if (latest) options.sendStage('stage.desktop.update', latest);
  });
  try {
    options.session.on('lock-screen', lock);
    options.session.on('unlock-screen', unlock);
  } catch (error) {
    options.session.removeListener('lock-screen', lock);
    options.session.removeListener('unlock-screen', unlock);
    stopReady();
    stopLoop();
    throw error;
  }
  let stopped = false;
  return () => {
    if (stopped) return;
    stopped = true;
    stopReady();
    options.session.removeListener('lock-screen', lock);
    options.session.removeListener('unlock-screen', unlock);
    stopLoop();
  };
}
