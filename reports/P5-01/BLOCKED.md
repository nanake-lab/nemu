# P5-01 — initial wire contract approval

## Blocking condition

AGENTS §2 requires an ADR for changes in core/contracts; AGENTS §7 and
CONTRACTS' modification rules require user approval of a new contract. The
initial exact byte layout is now proposed in ADR 0007, not adopted. W01-D01
records the pending contract review. C01/C02 are not reopened.

## Work completed without that approval

- Selected P5-01 after P0-06 PR #3 was merged and P0-02 was already merged;
  checked that no other active Codex task or open PR owned these directories.
- Prepared the message/source inventory and a clearly marked review-only proto
  candidate outside the authoritative contracts directory.
- Implemented independent Rust finite-value, bounded nanosecond interval and
  count helpers; no serialization, authority storage or runtime connection.
- Ran Rust workspace static checks and tests; evidence is in verification/.

## Options

1. Approve ADR 0007's explicit encoding policy, then continue P5-01's full
   source-mapped schemas, generators, decoders and three-language golden vectors.
2. Request specific changes to the endian, header, presence or hash-domain
   proposal before it becomes a wire contract.
3. Defer the contract and keep P5-01 incomplete. Its dependent cards cannot start.

Approval alone does not complete P5-01 or authorize its PR to merge before all
acceptance criteria and CI pass. The missing upstream growth specifications and
W09/W10 conflict remain separate. The next task in this chat remains P5-01.
