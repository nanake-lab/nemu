"""Real launcher/child process fixture; stdlib only, no GPU imports."""
import json
import os
from pathlib import Path
import subprocess
import sys
import time
base = Path(sys.argv[1])
mode = sys.argv[2]
allocation = bytearray((64 if mode == 'root' else 32) * 1024 * 1024)
for offset in range(0, len(allocation), 4096):
    allocation[offset] = 1
children = []
if mode == 'root':
    children.append(subprocess.Popen([sys.executable, __file__, str(base), 'child']))
    with (base / 'events.jsonl').open('w', encoding='utf-8') as stream:
        stream.write(json.dumps({'event': 'imports_ready', 'pid': os.getpid(), 'parent': os.getppid()}) + '\n')
(base / (mode + '.json')).write_text(json.dumps({'pid': os.getpid(), 'parent': os.getppid(), 'allocated': len(allocation)}), encoding='utf-8')
while not (base / 'finish').exists():
    time.sleep(0.1)
for child in children:
    child.wait(timeout=10)
