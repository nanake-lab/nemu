import type { Anchor, DndState } from '@nemu/contracts';

export interface DesktopSettingsV1 {
  version: 1;
  general: {
    openAtLogin: boolean;
    dnd: DndState;
  };
  shortcuts: {
    summonPanel: string;
    pushToTalk: string;
  };
  stage: {
    displayId?: number | undefined;
    anchor: Anchor;
  };
  supervisor: {
    autoStart: boolean;
    processes: Record<string, { enabled: boolean; customPath?: string | undefined }>;
  };
}

export type DesktopSettings = DesktopSettingsV1;
export const CURRENT_SETTINGS_VERSION = 1;

export function getDefaultSettings(): DesktopSettings {
  return {
    version: 1,
    general: {
      openAtLogin: false,
      dnd: 'none',
    },
    shortcuts: {
      summonPanel: 'CommandOrControl+Shift+N',
      pushToTalk: 'CommandOrControl+Shift+Space',
    },
    stage: {
      anchor: { kind: 'free', x: 0, y: 0 },
    },
    supervisor: {
      autoStart: true,
      processes: {
        'nemu-core.exe': { enabled: true },
        'irl-worker': { enabled: true },
        'drive-worker': { enabled: true },
        'model-adapter': { enabled: true },
        'voice-sidecar': { enabled: true },
        'codex-bridge': { enabled: true },
        'office-connector': { enabled: true },
      },
    },
  };
}

export function migrateSettings(raw: unknown): DesktopSettings {
  if (!raw || typeof raw !== 'object') {
    return getDefaultSettings();
  }

  const obj = raw as Record<string, unknown>;
  const version = typeof obj.version === 'number' ? obj.version : 0;

  let current = obj;

  if (version === 0) {
    // Migration from v0 (unversioned legacy / raw dictionary) to v1
    const defaults = getDefaultSettings();
    const generalRaw = (typeof current.general === 'object' && current.general !== null)
      ? (current.general as Record<string, unknown>)
      : {};
    const shortcutsRaw = (typeof current.shortcuts === 'object' && current.shortcuts !== null)
      ? (current.shortcuts as Record<string, unknown>)
      : {};

    current = {
      version: 1,
      general: {
        openAtLogin: typeof generalRaw.openAtLogin === 'boolean'
          ? generalRaw.openAtLogin
          : defaults.general.openAtLogin,
        dnd: typeof generalRaw.dnd === 'string'
          ? (generalRaw.dnd as DndState)
          : defaults.general.dnd,
      },
      shortcuts: {
        summonPanel: typeof shortcutsRaw.summonPanel === 'string'
          ? shortcutsRaw.summonPanel
          : defaults.shortcuts.summonPanel,
        pushToTalk: typeof shortcutsRaw.pushToTalk === 'string'
          ? shortcutsRaw.pushToTalk
          : defaults.shortcuts.pushToTalk,
      },
      stage: defaults.stage,
      supervisor: defaults.supervisor,
    };
  }

  // Ensure all required fields exist for v1
  const defaults = getDefaultSettings();
  const v1 = current as unknown as DesktopSettingsV1;

  return {
    version: 1,
    general: {
      openAtLogin: v1.general?.openAtLogin ?? defaults.general.openAtLogin,
      dnd: v1.general?.dnd ?? defaults.general.dnd,
    },
    shortcuts: {
      summonPanel: v1.shortcuts?.summonPanel ?? defaults.shortcuts.summonPanel,
      pushToTalk: v1.shortcuts?.pushToTalk ?? defaults.shortcuts.pushToTalk,
    },
    stage: {
      displayId: v1.stage?.displayId,
      anchor: v1.stage?.anchor ?? defaults.stage.anchor,
    },
    supervisor: {
      autoStart: v1.supervisor?.autoStart ?? defaults.supervisor.autoStart,
      processes: {
        ...defaults.supervisor.processes,
        ...(v1.supervisor?.processes ?? {}),
      },
    },
  };
}
