import { existsSync, readFileSync, writeFileSync, mkdirSync, copyFileSync, renameSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { app } from 'electron';
import {
  type DesktopSettings,
  getDefaultSettings,
  migrateSettings,
} from './migrations.js';

export class SettingsStore {
  private filePath: string;
  private settings: DesktopSettings;

  constructor(customPath?: string) {
    if (customPath) {
      this.filePath = customPath;
    } else {
      const scaffoldData = process.env.NEMU_SCAFFOLD_USER_DATA;
      if (scaffoldData) {
        this.filePath = join(scaffoldData, 'settings.json');
      } else {
        try {
          // On Windows, resolve %APPDATA%/Nemu/settings.json
          const appData = app.getPath('appData');
          this.filePath = join(appData, 'Nemu', 'settings.json');
        } catch {
          this.filePath = join(process.cwd(), 'settings.json');
        }
      }
    }
    this.settings = this.load();
  }

  public getPath(): string {
    return this.filePath;
  }

  public get(): DesktopSettings {
    return { ...this.settings };
  }

  public update(updater: (current: DesktopSettings) => Partial<DesktopSettings>): DesktopSettings {
    const changes = updater(this.get());
    this.settings = migrateSettings({ ...this.settings, ...changes });
    this.save();
    return this.get();
  }

  public load(): DesktopSettings {
    try {
      if (!existsSync(this.filePath)) {
        const defaults = getDefaultSettings();
        this.settings = defaults;
        this.save();
        return defaults;
      }
      const rawText = readFileSync(this.filePath, 'utf-8');
      const parsed = JSON.parse(rawText) as unknown;
      const migrated = migrateSettings(parsed);
      this.settings = migrated;
      return migrated;
    } catch (err) {
      // Backup corrupted settings before falling back to defaults
      try {
        if (existsSync(this.filePath)) {
          const backupPath = `${this.filePath}.bak.${Date.now()}`;
          copyFileSync(this.filePath, backupPath);
          console.warn(`[SettingsStore] Corrupt settings backed up to ${backupPath}:`, err);
        }
      } catch (backupErr) {
        console.warn('[SettingsStore] Failed to backup corrupt settings file:', backupErr);
      }
      const defaults = getDefaultSettings();
      this.settings = defaults;
      return defaults;
    }
  }

  public save(): void {
    try {
      const dir = dirname(this.filePath);
      if (!existsSync(dir)) {
        mkdirSync(dir, { recursive: true });
      }
      // True atomic write via temporary file + renameSync
      const tempPath = `${this.filePath}.${process.pid}.${Date.now()}.tmp`;
      writeFileSync(tempPath, JSON.stringify(this.settings, null, 2), 'utf-8');
      renameSync(tempPath, this.filePath);
    } catch (error) {
      console.error(`Failed to save settings to ${this.filePath}:`, error);
    }
  }
}
