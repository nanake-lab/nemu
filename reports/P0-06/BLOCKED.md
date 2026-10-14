# P0-06 — required user decisions

## W00-C01: single-root consolidation

Verified in original DOCX XML as well as the readable conversions:

- `Echo_Core_v0.6.4_Frozen_Architecture.docx` §§7,7.1,16.2,16.9 admits delayed single-root consolidation under strict conditions.
- `Ada Echo Core Math define.docx` §§9,18 I20,19,20 C01 requires more than one causally independent root for a new nonzero θ update.
- `Echo Core Implementation Plan.docx` §17 follows the latter rule.

The repository's stated precedence cannot be silently changed. Readable conversions, originals and existing ADRs were checked; no recorded approval settling this conflict was found. This is not resolved by enabling either branch. v1.0 stays NO-GROWTH, as already approved.

Options: (1) explicitly approve the newer multi-root-only closure as superseding the older single-root branch; (2) explicitly approve the bounded v1.0 baseline and defer this excluded growth decision to v1.x; (3) provide the corrected/upstream authoritative text first. Details and consequences: [proposed ADR 0006](../../docs/adr/0006-single-root-spec-conflict.md).

## W00-C02: system-test count

Original System Plan §29 enumerates SYS01–SYS25; §32 says 24 system tests. All 25 IDs are preserved, including SYS25 in the M2 subset. Confirm that the prose count is a typo; no test is removed or threshold relaxed.

## Independent work completed

Five source hashes, E1–E14 register, toolchain and SQLite probe, scope/acceptance routing, specification mapping and report are complete. Source documents unchanged. Missing Review_and_Remediation remains the existing E12 item and is not newly treated as a v1.0 dependency. These decisions block P0-06 acceptance/merge, not the independent evidence already gathered.
