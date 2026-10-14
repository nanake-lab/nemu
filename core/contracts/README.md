# Echo Core IDL / CanonicalV1

Approved byte contract: `docs/adr/0007-initial-idl-canonical-encoding.md`.
The four `.proto` files are the sole source of fields, enum values, frozen type
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
