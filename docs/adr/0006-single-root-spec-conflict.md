# ADR 0006 — Single-root consolidation specification conflict

Status: **PROPOSED — requires user approval; not an adopted architecture change**.
Date: 2026-10-14. Task: P0-06 / W00-C01.

## Context

Original files are identified by filename and hash in `docs/traceability/baseline.md`. Frozen Architecture (F) §§7,7.1,16.2,16.9 admits a tightly constrained single-root consolidation branch. Operators v1.0 (O) §§9,18 I20,19,20 C01 and System Plan (S) §17 prohibit a new nonzero θ update without more than one independent root. README §1 ranks frozen ontology above later explicit closures; the agent cannot determine whether this particular earlier clause was superseded by user intent. Both wordings occur in the DOCX originals.

## Decision requested (none adopted)

Choose one:

1. Confirm O's multi-root-only closure supersedes F's single-root eligibility for future growth. Preserve the historical original and record the explicit precedence exception in the register.
2. Defer the growth conflict to v1.x, explicitly approving W00's v1.0 baseline with this unresolved, excluded branch. Future G/C work remains blocked until a separate decision and missing upstream review.
3. Provide the intended corrected/upstream specification and reconcile it before accepting W00. No new rule is inferred meanwhile.

## Interim behavior and consequences

Existing approved v1.0 behavior is unchanged: empty R, genesis θ, identity G/C, audited NO-GROWTH/GROWTH_NOT_ENABLED_V1. No single-root or multi-root growth is implemented in P0-06. Option 1 affects future growth acceptance and must not silently delete original 16.2/16.9 evidence. Option 2 permits only an explicitly bounded v1.0 baseline, not a claim that the full specifications are consistent. Option 3 delays the baseline pending authoritative text. No original document, threshold, IDL or runtime code is changed by this proposal.

## Status update — 2026-10-19

**SUPERSEDED by ADR 0009.** The v2.0 core (`docs/echo-core/echo-core-v2.0-first-principles.md`) has no single-root or multi-root qualification, so W00-C01 is dissolved rather than decided between options 1–3. The text above is kept as history.
