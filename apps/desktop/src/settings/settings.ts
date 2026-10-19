import { existsSync, readFileSync, writeFileSync, mkdirSync } from 'node:fs';
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
      try {
        const userData = app.getPath('userData');
        this.filePath = join(userData, 'settings.json');
      } catch {
        // Fallback if app is not ready or in pure Node environment
        this.filePath = join(process.cwd(), 'settings.json');
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
    } catch {
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
      writeFileSync(this.filePath, JSON.stringify(this.settings, null, 2), 'utf-8');
    } catch (error) {
      console.error(`Failed to save settings to ${this.filePath}:`, error);
    }
  }
}
