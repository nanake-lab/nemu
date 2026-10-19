import { appendFileSync, existsSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { app } from 'electron';

export class DesktopLogger {
  private logDir: string;
  private logFile: string;

  constructor(customDir?: string) {
    if (customDir) {
      this.logDir = customDir;
    } else {
      const scaffoldData = process.env.NEMU_SCAFFOLD_USER_DATA;
      if (scaffoldData) {
        this.logDir = join(scaffoldData, 'logs');
      } else {
        try {
          this.logDir = join(app.getPath('appData'), 'Nemu', 'logs');
        } catch {
          this.logDir = join(process.cwd(), 'logs');
        }
      }
    }

    try {
      if (!existsSync(this.logDir)) {
        mkdirSync(this.logDir, { recursive: true });
      }
    } catch (err) {
      console.warn(`Failed to create log dir ${this.logDir}:`, err);
    }

    this.logFile = join(this.logDir, 'app.log');
  }

  public getLogDir(): string {
    return this.logDir;
  }

  public getLogFilePath(): string {
    return this.logFile;
  }

  public write(level: 'info' | 'warn' | 'error' | 'debug', message: string, detail?: Record<string, unknown>): void {
    const entry = {
      timestamp: new Date().toISOString(),
      pid: process.pid,
      level,
      message,
      ...(detail ? { detail } : {}),
    };

    const line = JSON.stringify(entry) + '\n';
    try {
      appendFileSync(this.logFile, line, 'utf-8');
    } catch {
      // Avoid crashing if disk write fails
    }

    if (level === 'error') {
      console.error(`[Desktop] ${message}`, detail ?? '');
    } else if (level === 'warn') {
      console.warn(`[Desktop] ${message}`, detail ?? '');
    }
  }

  public info(message: string, detail?: Record<string, unknown>): void {
    this.write('info', message, detail);
  }

  public warn(message: string, detail?: Record<string, unknown>): void {
    this.write('warn', message, detail);
  }

  public error(message: string, detail?: Record<string, unknown>): void {
    this.write('error', message, detail);
  }

  public debug(message: string, detail?: Record<string, unknown>): void {
    this.write('debug', message, detail);
  }
}
