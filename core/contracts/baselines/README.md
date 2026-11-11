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
| `p5-32/` | P5-32 (PR #52), published IDL/golden unchanged through main `91b31ed`; frozen by P5-22 before edits |
| `p5-36/` | Pre-P5-68 published IDL and golden vectors at main `a7e8ae8790d297ac697956f6f1256d2e2b9403d5` (includes P5-36 and subsequent contract additions); frozen before P5-68 edits |
| `p5-68/` | Published IDL and golden vectors at main `8839dcae73c98437f61222f49d08cafd9ba8f4e6`; frozen before P5-37 edits |
