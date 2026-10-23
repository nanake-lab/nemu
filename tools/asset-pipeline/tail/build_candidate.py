"""Compatibility entry for the standard tail stage."""
from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from blender_common import run
from build_tail import execute


if __name__ == "__main__":
    run("tail-candidate", execute)
