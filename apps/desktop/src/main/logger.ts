import { appendFileSync, existsSync, mkdirSync, renameSync, statSync, unlinkSync } from 'node:fs';
import { join } from 'node:path';
import { app } from 'electron';

export class DesktopLogger {
  private logDir: string;
  private logFile: string;
  private maxSizeBytes = 5 * 1024 * 1024; // 5 MB per file
  private writesSinceCheck = 0;

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

  private checkRotation(): void {
    try {
      if (existsSync(this.logFile)) {
        const stat = statSync(this.logFile);
        if (stat.size >= this.maxSizeBytes) {
          const backupFile = join(this.logDir, 'app.1.log');
          if (existsSync(backupFile)) {
            unlinkSync(backupFile);
          }
          renameSync(this.logFile, backupFile);
        }
      }
    } catch {
      // Avoid failing log operations on rotation errors
    }
  }

  public write(level: 'info' | 'warn' | 'error' | 'debug', message: string, detail?: Record<string, unknown>): void {
    this.writesSinceCheck++;
    if (this.writesSinceCheck > 100) {
      this.writesSinceCheck = 0;
      this.checkRotation();
    }

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
