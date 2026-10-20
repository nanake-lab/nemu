import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { writeFileSync } from 'node:fs';
import {
  migrateSettings,
  getDefaultSettings,
  SettingsStore,
} from './index.js';

describe('Settings Migration and Store', () => {
  let tempDir: string;

  beforeEach(async () => {
    tempDir = await mkdtemp(join(tmpdir(), 'nemu-settings-test-'));
  });

  afterEach(async () => {
    await rm(tempDir, { recursive: true, force: true });
  });

  it('provides default settings when raw input is null or empty', () => {
    const defaults = getDefaultSettings();
    expect(migrateSettings(null)).toEqual(defaults);
    expect(migrateSettings({})).toEqual(defaults);
    expect(migrateSettings('invalid')).toEqual(defaults);
  });

  it('migrates unversioned v0 settings to v1', () => {
    const v0 = {
      general: {
        openAtLogin: true,
        dnd: 'busy',
      },
      shortcuts: {
        summonPanel: 'CommandOrControl+Alt+N',
      },
    };

    const migrated = migrateSettings(v0);
    expect(migrated.version).toBe(1);
    expect(migrated.general.openAtLogin).toBe(true);
    expect(migrated.general.dnd).toBe('busy');
    expect(migrated.shortcuts.summonPanel).toBe('CommandOrControl+Alt+N');
    expect(migrated.shortcuts.pushToTalk).toBe('CommandOrControl+Shift+Space');
    expect(migrated.supervisor.autoStart).toBe(true);
  });

  it('loads, updates and saves settings in SettingsStore', () => {
    const settingsPath = join(tempDir, 'settings.json');
    const store = new SettingsStore(settingsPath);

    expect(store.get().version).toBe(1);
    expect(store.get().general.openAtLogin).toBe(false);

    store.update((current) => ({
      general: {
        ...current.general,
        openAtLogin: true,
        dnd: 'quietHours',
      },
    }));

    expect(store.get().general.openAtLogin).toBe(true);
    expect(store.get().general.dnd).toBe('quietHours');

    // Reload from disk to verify persistence
    const reloadedStore = new SettingsStore(settingsPath);
    expect(reloadedStore.get().general.openAtLogin).toBe(true);
    expect(reloadedStore.get().general.dnd).toBe('quietHours');
  });

  it('recovers gracefully from corrupted json file', () => {
    const settingsPath = join(tempDir, 'corrupted.json');
    writeFileSync(settingsPath, '{ invalid json ...', 'utf-8');

    const store = new SettingsStore(settingsPath);
    expect(store.get()).toEqual(getDefaultSettings());
  });
});
