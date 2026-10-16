"""Keep local GPU/render measurements separate from the self-hosted CI runner."""

import csv
import io
import os
import subprocess


def require_runner_offline() -> None:
    if os.name != "nt":
        raise RuntimeError("P0-07 GPU measurements must run on the user's Windows GPU host")
    for name in ("Runner.Listener.exe", "Runner.Worker.exe"):
        result = subprocess.run(
            ["tasklist", "/FI", f"IMAGENAME eq {name}", "/FO", "CSV", "/NH"],
            check=True,
            capture_output=True,
            text=True,
        )
        for row in csv.reader(io.StringIO(result.stdout)):
            if row and row[0].casefold() == name.casefold():
                raise RuntimeError(
                    f"{name} is online; stop runner before local GPU/render measurement"
                )
