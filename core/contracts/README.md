# Echo Core IDL / CanonicalV1

Approved byte contract: `docs/adr/0007-initial-idl-canonical-encoding.md`.
The `.proto` files in this directory are the sole source of fields, enum values, frozen type
tags, required presence, bounds, set keys and variant constraints. `@message`
includes the source specification pointer; `@rules` expresses source-required
cross-field constraints. All R/G objects are types only; no runtime is wired here.

## Generate and verify (PowerShell, repository root)

```powershell
python core/contracts/generate.py
python core/contracts/generate.py --check
pnpm test:core-contracts
./reports/P5-01/verify-full.ps1
```

Conformance builds Rust and TypeScript probes and uses the locked development
environment in `core/contracts/uv.lock` for official Protobuf interoperability.
It never updates golden expectations. `freeze_vectors.py` is an explicit
development operation: its changes require byte review and all conformance
checks before acceptance. Fixture limits are test budgets, not deployment policy.

## APIs

- Rust: `core_types::generated::*`, `WireMessage::{canonical_bytes,protobuf_bytes,
  from_canonical,from_protobuf}`. `contract::Schema::digest` adds the approved
  object-type domain. Nonfinite floats are rejected before serde can erase them.
- TypeScript: browser-safe generated types/schema in the `EchoCore` namespace
  from `@nemu/contracts`. The separate generated `echo/index.ts` codec entry is
  for Node/Electron main: typed `encodeCanonical`, `decodeCanonical`,
  `encodeProtobuf`, `decodeProtobuf` and `inputSchema`. It is not loaded by the
  shared browser contract barrel. 64-bit numbers are `bigint`, bytes are `Uint8Array`.
- Python: each worker's generated `echo.types` contains frozen dataclasses and
  enums. `echo.typed_api.{to_value,from_value}` bridges DTOs to the validating
  `echo.runtime` codecs. 64-bit values remain Python integers; bytes remain bytes.

All entry points require explicit byte/item/depth budgets. Wire decoders reject
unknown fields, enum zero/unknown values, missing required scalar/message fields,
duplicate singulars, malformed UTF-8, nonfinite floats, overflow and over-budget
input. A missing protobuf repeated field denotes an empty vector; native objects
must supply required vectors. Protobuf accepts packed numeric vectors and any
field-group order, while preserving order within ordered vectors.

CanonicalV1 rejects trailing bytes, negative-zero encodings, unsorted/duplicate
set keys and invalid presence bytes. Transport negative zero normalizes to +0.
Nested messages contain their own NCV1 headers; hashes include the full root
header. Tags are independent frozen strings, not names derived from protobuf.

## Scope and lifecycle

ObjectRef denotes an immutable recoverable object under S §§6–7, including its
digest, type, resource version and byte length. Opaque semantic resources retain
their explicit version; this card does not define missing Review_and_Remediation
mechanisms, evaluate qualifications, authorize actions or resolve referenced
objects. Those checks remain with their owning cards. `AuditExtension` has a
separate audit schema tag/version; it cannot silently become causal state.

Before N3, revisions require backup and migration documentation. Genesis signature
locks the encoding; subsequent changes require a new encoding version and migration.
Changing a frozen type tag always means a new encoding version. All tests use
synthetic data in memory/build scratch directories and never open lineage stores.

## v2.0 state and records (P5-21)

`ledger.proto`, `law.proto` and `records.proto` add the Echo Core v2.0 types: interpretation
records, event attributes, R (tendencies, evidence groups), θ, the change ledger L, growth
commands, the law version and the genesis manifest. Changed messages are successors with new
frozen tags (`VersionedEnvironmentV2`, `ProposalBindingV2`, `FastStateSnapshotV2`,
`CoreTransactionV2`, `CoreCommitResultV2`, `ModulationEnvelopeV2`, `LineageHeaderV2`, and the
two carriers of the new binding); every published message keeps its exact bytes.

- `// @superseded <successor or -> | <reason>` in a message body, before an enum, or after an
  enum value marks it superseded. It stays decodable to read old data and for tests; no
  runtime path may produce it (`WireMessage::SUPERSEDED`, `<Enum>::is_superseded`,
  `generated::SUPERSEDED_MESSAGES`). Live messages may not embed superseded types.
- `// @supersedes <predecessor>` marks a successor. It keeps each predecessor field number
  identical or `reserved`, and numbers new fields above the predecessor's maximum.
- `check_compat.py` (run by `verify.py`) compares the IDL with every frozen baseline in
  `baselines/` and fails on any reused field number, enum value or type tag, or any change to
  a frozen golden vector; its self-test proves each rule fires. `--ref <git-ref>` also
  compares with the `.proto` files at a ref.
  The gate also checks the current IDL against itself, as the next contract card will
  freeze it as a baseline. Self-test fixtures never rewrite a published field: rules on
  successors are exercised on a synthetic chain `SelfTestV1 → V2 → V3`, and each mutation
  counts only if the rule it targets fires. `--baselines <dir>` points the gate at another
  baseline directory.
- `freeze_vectors.py` is append-only: it re-verifies every frozen vector byte for byte with the
  Rust authority before adding vectors for messages that have none yet and the targeted P5-21
  vectors in `fixture_inputs.py`.
- Semantic checks the codec cannot express (unit d_k, μ > 0, π ∈ (−1, 1), law parameter
  inequalities, genesis consistency) are in `core_types::semantic`.

## P5-22 terminal successors and required oneof

The ReadoutV2.body Protobuf oneof is also a CanonicalV1 oneof under ADR 0007 §6:
ascending field position, selected member number (u32 LE), then that value. No
per-member presence markers are written. The generator currently accepts only
required, annotated, contiguous oneof members and fails closed on unsupported
syntax. All prior fields follow their unchanged encoding paths.

The descriptor records member ownership and the required group. Validators reject
missing/multiple arms; canonical decoders reject unknown member numbers and
trailing data. The strict Protobuf decoder retains both distinct arms until
validation, then rejects multiple selection instead of last-value-wins.
check_compat compares membership in both field metadata and rules and its targeted
self-tests exercise number reuse and changed membership. Text-level regressions
in test_oneof_schema.py are run by verify.py. P5-32's published schema/golden prefix
is now frozen in baselines/p5-32/; --ref origin/main additionally covers published
content since that baseline.

Readout/RunState/Query/QueryResult/ApprovalRequest/ApprovalDecision have successors;
old bytes and decoders remain. BodyReadout v1 remains a supported compatibility arm
with a production-superseded comment, so it is intentionally not machine-marked as
an illegal dependency of the new ReadoutV2. P5-19 produces only BodyReadoutV2.

See reports/P5-22/REPORT.md for the paired-codec changes authorized by the user,
the frozen-byte hash comparison, Claude's P5-01 review and migration boundaries.
