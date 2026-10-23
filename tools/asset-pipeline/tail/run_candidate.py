"""Review-only complete pipeline using P2-05's tail stage; lint remains unchanged."""
from __future__ import annotations

import sys
import subprocess
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import pipeline


if __name__ == "__main__":
    subprocess.run(["pwsh", "-NoProfile", "-Command",
                    "$busy=@(Get-CimInstance Win32_Process | Where-Object { $_.Name -match '^(blender|electron|Runner\\.Worker|cargo|rustc)\\.exe$' -or ($_.Name -match '^(python|node|uv)\\.exe$' -and $_.CommandLine -match 'measure|benchmark|run_all|voice_sidecar.server|align_user|context_references') }); if($busy.Count){$busy | Select-Object Name,ProcessId | ConvertTo-Json; exit 1}"], check=True)
    pipeline.STEPS["tail"] = "tail/build_candidate.py"
    pipeline.main()
