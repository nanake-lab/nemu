# P0-07 validation after latest main sync — 2026-10-19

- Branch: `task/P0-07-carrier-probe`
- Latest `origin/main` merged: `1e9157e` (contains PR #25 rules and the P2-02 closeout).
- Merge commit: `e965f10`.
- Python tools reused from the isolated probe venv at `D:\Dev\nemu\workers\model-adapter\probe\.venv`; tests ran against this branch worktree.

| Check | Result | Evidence |
| --- | --- | --- |
| CPU mechanism tests (`pytest test_injection.py`) | passed, 4 tests | `pytest.log` |
| Tiny CPU path probe | exited 0; zero-path and changed-cache errors 0; cache discarded | `tiny-cpu-path.log`, `tiny-cpu-path.json` |
| Ruff | passed | `ruff.log` |
| Mypy strict | passed, 14 source files | `mypy.log` |

No pretrained-model or GPU benchmark was rerun. The Windows runner was online, so GPU/render measurements were not started. The tiny CPU fixture is marked `measured-not-accepted` and does not establish real model, concurrent load, latency, or direction acceptance. Existing P0-07 blockers remain: final carrier selection, production numerical tolerance, Nemu VRAM attribution, and direction quality evidence.
