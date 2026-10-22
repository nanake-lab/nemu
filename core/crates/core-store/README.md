# W02 storage format 1

`Store<L>` is the single authority writer. `L` is the sealed `Rehearsal`, `Life`, or
`Experiment` marker. `Reader<L>` checks the stored lineage before exposing any
records. An opened writer holds an OS file lock for its entire lifetime; a
non-cloneable `CommitCapability<L>` can be taken once per opening. Workers receive
generated proposals and permitted read views, not `Store`, the capability, or its
SQLite connection. Worker OS isolation remains the launcher/gateway's responsibility.

All tests use newly owned `TemporaryRoot` directories. This type has no constructor
from an existing path. Production APIs resolve only the designated local directories;
EXPERIMENT has no production path. Initialization refuses existing directories. LIFE
creation requires the explicit genesis API and the matching generated manifest digest.
P5-15 must validate the user's authorization, complete resources, and enabled runtime
stages before invoking birth. W02 does not execute birth in its tests.

## Data and durability

The database is `store.sqlite`, with WAL, FULL synchronization, foreign keys, an
explicit busy timeout and page limit. Production requires a local fixed NTFS volume.
Windows directory handles include write access and BACKUP_SEMANTICS so a failed
directory flush is an error. Win32 volume queries are the sole localized unsafe
boundary; the rest of the crate denies unsafe code. Linux supports temporary tests.

The table groups follow S §7, with R/θ/L represented by generated CanonicalV1
objects, reference indexes and snapshot pointers. SQLite integer conversion is
checked; unrepresentable positions/anchors are rejected instead of truncated.
History, decisions, references and ledger rows have append-only triggers.

Objects are addressed by the core-types type-domain-separated SHA-256 digest.
Their logical bytes are generated CanonicalV1, never Protobuf. Object writes use a
same-directory temporary file, read-back verification, file sync, non-replacing
atomic publication and parent-directory sync. Reusing an existing object verifies
and syncs it again, including the parent, covering an interrupted earlier rename.
Readers recheck type, length and digest. Published reference closure loss stops recovery.

R uses a physical `NSI1` index: length-prefixed canonical header/count, followed by
length-prefixed generated `ObjectRef` values for canonical `TendencyV2` objects.
Reassembly must equal the generated `RSnapshotV2` byte sequence and digest exactly.
This is physical segmentation, not a new R message or semantic encoding. Unchanged
tendencies reuse the same files. Θ and individual L entries use their generated
types directly. Full transaction manifests and SQL snapshot/ledger indexes preserve
the state pointers and append-only L history; no second psychological state is added.

Prepared objects and transactions hold closure pins. GC and publication share their
retention lock, and GC retains all D/published history. Unpublished orphan files can
be removed only after their pins are released. Readers can only resolve objects
published by an authority transaction; receipt of D alone does not expose it as H.

## Commit protocol

1. `ingest` validates and durably appends D and its object closure before returning
   `RECEIVED_DURABLE`. Queue/byte capacity failures return an error before success.
2. The runtime captures the generated `ProposalBindingV2` with `bind` before
   dispatch. All state roots and bound resources must occur in its read set;
   `read_bound` rejects undeclared reads. The initial base identity is the canonical
   lineage-header digest. Later bases are the last commit digest.
3. Operators produce the generated `CoreTransactionV2`. `stage` checks the exact
   original binding and retains the canonical state/ledger closure. W02 does not
   calculate Φ/A/M/F/G/C or certify their mathematical results. The runtime supplies
   all projected/z/context digests and the actual `DegradedStatus`.
4. `commit` checks the capability, head, frontier, revisions, complete binding and
   retained object bytes again. One SQLite transaction publishes H, x, R, θ, L, Q,
   U, accepted results, concepts, indexes, time anchor, position and head. The
   returned commit digest identifies the canonical transaction. No stale proposal
   is automatically rebased. Duplicate/conflict settlement leaves the state snapshot
   and anchor unchanged while advancing the durable ingress decision prefix.

The P5-32 evidence index records accepted `EvidenceGroupV2` versions and allocations.
Queries choose each group's latest accepted version at an explicit commit, so removing
an allocation cannot resurrect an older association. Ledger/evidence queries reject
unknown prefixes and insufficient result budgets rather than reporting false absence.
Higher-level policy qualification and fact-query semantics remain their owning cards.

## Recovery receipts and AT04

Redundant receipts live in `.<store-name>.w02-receipts-v1/` beside the store directory,
under the same local parent. `d/` and `commits/` contain position-named canonical
`ObjectRef` files. These are durability witnesses to existing D/commit records, not
an alternative authority or mutable budget. A successful call requires both SQLite
commit and the matching receipt to be durable. Missing receipts for a verified SQL
tail can be reconstructed after a crash before acknowledgement. A receipt that is
ahead of, or disagrees with, the database causes a recovery error.

Consequently copying a branch to a new location cannot open as the original, a
second writer cannot open it, and replacing an old database at its original location
cannot refund acknowledged ledger entries. Ordinary opening never resets receipts.
Backups/restores must retain this receipt namespace and use W14 for tail recovery or
an explicitly registered lossy branch. Rolling back the entire device including all
independent recovery evidence is outside what local software can detect; external
backup provenance and recovery authorization belong to P5-15. No restore/import API
is silently supplied here.

Crash injection exits the process without destructors at file-write/sync/rename/
directory-sync, G staging, C staging, ledger insertion, before SQLite commit, after
SQLite commit/before receipt, and after complete publication. These are process-crash
and disk-error tests; they do not claim a physical power-cut or drive-firmware test.

## Checks

From `core/`:

```powershell
cargo test -p core-store --locked -- --test-threads=1 --nocapture
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

W02 tests stay in this crate's authorized directory. Acceptance-ID mappings and
machine-readable results are in `reports/P5-02/` and `docs/traceability/P5-02.md`;
central W16 registration is requested separately. No test opens production paths.
