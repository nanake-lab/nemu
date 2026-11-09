"""Keyless Windows process-lifetime diagnostic; no network or model access."""
from __future__ import annotations

import argparse
import ctypes
import json
import os
import subprocess
import sys
import time
from ctypes import wintypes
from datetime import datetime, timezone
from pathlib import Path


def utc() -> str:
    return datetime.now(timezone.utc).isoformat()


def write(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')


def in_job() -> bool:
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.GetCurrentProcess.restype = wintypes.HANDLE
    kernel.IsProcessInJob.argtypes = [wintypes.HANDLE, wintypes.HANDLE, ctypes.POINTER(wintypes.BOOL)]
    answer = wintypes.BOOL()
    if not kernel.IsProcessInJob(kernel.GetCurrentProcess(), None, ctypes.byref(answer)):
        raise ctypes.WinError(ctypes.get_last_error())
    return bool(answer.value)


def beat(output: Path, seconds: int) -> None:
    write(output / 'started.json', {'utc': utc(), 'pid': os.getpid(), 'parent_pid': os.getppid(),
          'in_windows_job': in_job(), 'seconds': seconds, 'key_access': False, 'network': False})
    started = time.monotonic()
    with (output / 'heartbeat.jsonl').open('x', encoding='utf-8') as stream:
        while time.monotonic() - started < seconds:
            stream.write(json.dumps({'utc': utc(), 'elapsed': time.monotonic() - started}) + '\n')
            stream.flush()
            time.sleep(1)
    write(output / 'terminal.json', {'utc': utc(), 'status': 'completed',
          'elapsed': time.monotonic() - started})


def launch(output: Path, pwsh: str) -> None:
    output.mkdir(exist_ok=False)
    write(output / 'launcher.json', {'utc': utc(), 'pid': os.getpid(), 'in_windows_job': in_job()})
    for name, flags in [('attached_console', subprocess.CREATE_NEW_CONSOLE),
                        ('detached', subprocess.DETACHED_PROCESS | subprocess.CREATE_NEW_PROCESS_GROUP),
                        ('breakaway', subprocess.DETACHED_PROCESS | subprocess.CREATE_BREAKAWAY_FROM_JOB)]:
        folder = output / name
        folder.mkdir()
        if name == 'attached_console':
            command = [pwsh, '-NoProfile', '-File', str(Path(__file__).with_name('process_lifetime_entry.ps1')),
                       '-OutputDirectory', str(folder)]
        else:
            command = [sys.executable, '-B', str(Path(__file__).resolve()), 'beat', '--output', str(folder)]
        startup = subprocess.STARTUPINFO()
        startup.dwFlags = subprocess.STARTF_USESHOWWINDOW
        startup.wShowWindow = 0  # Only our diagnostic helper; no user window.
        try:
            with (folder / 'stdout').open('xb') as stdout, (folder / 'stderr').open('xb') as stderr:
                child = subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=stdout,
                                         stderr=stderr, creationflags=flags, startupinfo=startup)
            write(folder / 'launch.json', {'utc': utc(), 'pid': child.pid, 'flags': flags, 'argv': command})
        except OSError as error:
            write(folder / 'launch-error.json', {'utc': utc(), 'winerror': error.winerror,
                  'error_class': type(error).__name__, 'flags': flags})


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=('launch', 'beat'))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--pwsh')
    parser.add_argument('--seconds', type=int, default=330)  # Cross the suspected five-minute boundary.
    args = parser.parse_args()
    if args.mode == 'launch':
        if not args.pwsh:
            parser.error('--pwsh required for launch')
        launch(args.output, args.pwsh)
    else:
        beat(args.output, args.seconds)
