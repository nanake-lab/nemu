# Published contract baselines

Each subdirectory freezes one published contract: its `.proto` files exactly as
merged, and `golden.json`, the entry count and SHA-256 of its golden vectors.
`check_compat.py` (run by `verify.py`) compares the current IDL with every
baseline and fails on any reused field number, enum value or type tag, or any
change to a frozen golden vector. Baselines are append-only: never edit one.

| Baseline | Source |
| --- | --- |
| `p5-01/` | P5-01 (PR #4), `core/contracts/*.proto` and `golden/` at main `cfd635f` |
| `p5-21/` | P5-21 (PR #36), `core/contracts/*.proto` and `golden/` at main `9042c59` |
