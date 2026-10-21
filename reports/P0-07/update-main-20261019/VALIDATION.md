# P0-07 post-main validation — 2026-10-19

- Branch: `task/P0-07-carrier-probe`
- Main base merged: `0208cdf` (PR #25)
- Merge commit: `2e78873`
- Test source: branch worktree `D:\Dev\nemu-p007`; Python tools reused from the existing isolated probe venv at `D:\Dev\nemu\workers\model-adapter\probe\.venv`.

| Check | Result | Evidence |
| --- | --- | --- |
| CPU mechanism tests (`pytest test_injection.py`) | passed, 4 tests | `pytest.log` |
| Tiny CPU path probe | exited 0 | `tiny-cpu-path.log`, `tiny-cpu-path.json` |
| Ruff | passed | `ruff.log` |
| Mypy strict | passed, 14 source files | `mypy.log` |

No pretrained-model or GPU benchmark was rerun in this pass. The Windows runner was online, so GPU/render measurements were not started. Existing P0-07 reports remain partial: final carrier choice, production numerical tolerance, Nemu VRAM attribution, and direction quality evidence are still unresolved. No thresholds were changed and no candidate was marked selected.
