# W02 storage format 1

`Store<L>` is the single authority writer. `L` is the sealed `Rehearsal`, `Life`, or
`Experiment` marker. `Reader<L>` checks the stored lineage before exposing any
records. An opened writer holds an OS file lock for its entire lifetime; a
non-cloneable `CommitCapability<L>` can be taken once per opening. Workers receive
generated proposals and permitted read views, not `Store`, the capability, or its
SQLite connection. Worker OS isolation remains the launcher/gateway's responsibility.

The test coordinator creates every sandbox through `TemporaryRoot`, which has no
public constructor from an existing path. Crash children receive a one-use permit
via stdin: the coordinator creates a random permit file inside its owned root,
containing the complete request. Before any lock or SQLite open, the child checks
the direct system-temp location, root name, absence of links, exact permit contents,
and consumes the permit. Arbitrary path environment variables are ignored. This
protects against accidental test invocation; it is not an OS boundary against a
malicious process with the same user's filesystem permissions. Production APIs resolve only the designated local directories;
EXPERIMENT has no production path. Initialization refuses existing directories. LIFE
creation requires the explicit genesis API and the matching generated manifest digest.
P5-15 must validate the user's authorization, complete resources, and enabled runtime
stages before invoking birth. W02 does not execute birth in its tests.

## Data and durability

The database is `store.sqlite`, with WAL, FULL synchronization, foreign keys, an
explicit busy timeout and page limit. Production requires a local fixed NTFS volume.
Windows file publication and volume queries are confined to `platform.rs`, the
only module allowing documented unsafe calls; the rest denies unsafe code. Linux
supports temporary tests. Windows primitives and assumptions are detailed below.

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

Independent readers capture immutable receipts before pinning a SQLite read
transaction. They verify the same object closure, canonical commit manifests, head,
fact/snapshot/ledger indexes and receipt prefix as the writer, within that snapshot.
SQL publication precedes each receipt: all captured receipts must therefore exist
in the later SQL snapshot. Receipts appended afterwards are not compared against an
older snapshot. The read transaction ends before returning the live reader; later
queries observe append-only publications. A genuinely old backup still fails.

Crash selection identifies an operation, an optional generated object tag and a
1-based matching occurrence. The matrix separately covers D, transaction manifests,
R's second tendency and index, existing-object re-sync, receipt publication and
repeated crashes during tail repair. Initialization and GC have separate assertions:
partial initialization fails closed; GC can only remove unreferenced orphans.
Detailed cases and actual exit traces are recorded under `reports/P5-02/review-1/`.
No finite injection matrix is described as covering every possible I/O or hardware
failure. Process exits bypass destructors but do not simulate physical power loss.

## Windows publication primitives and evidence boundary

- File contents: `File::sync_all()` invokes `FlushFileBuffers`; the file is open
  with write access, and a failure propagates. See [FlushFileBuffers](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-flushfilebuffers).
- Name publication: clear `FILE_ATTRIBUTE_TEMPORARY`, then use same-directory
  `MoveFileExW(MOVEFILE_WRITE_THROUGH)`. No `REPLACE_EXISTING`, `COPY_ALLOWED` or
  delayed operation is permitted. Microsoft documents waiting for the move to
  reach disk with write-through, and separately describes flushing copy/delete
  moves. This implementation never takes the cross-volume copy/delete path.
  See [MoveFileExW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw).
- Parent directory: obtain a read/write directory handle with
  `FILE_FLAG_BACKUP_SEMANTICS`, then `FlushFileBuffers`. This flag permits a
  directory handle; it does not itself prove persistence. See [CreateFileW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew).
- Publication ordering relies on the local NTFS move, the explicit write-through
  request and successful file/directory flushes, not merely a POSIX-like helper
  name. It assumes the OS/filesystem/device stack honors those operations. Calls
  that fail stop publication; no weaker filesystem fallback is attempted.
- Windows/NTFS tests establish successful calls, no-clobber behavior and recovery
  after process exit. They do not independently establish all namespace persistence
  premises across sudden power loss, controller write caches or faulty firmware.
  No physical power-cut certification is claimed. SQLite WAL/FULL has the analogous
  dependence on the storage stack honoring its sync requests.

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
