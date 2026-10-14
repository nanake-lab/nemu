# ADR 0007 — Initial IDL and CanonicalV1 encoding contract

Status: **PROPOSED — requires user approval; not implemented as a wire contract**.
Date: 2026-10-14. Task: P5-01 / W01. Change class: **契约变更**.

## Background

System Plan (S) §§2,6 and P5-01 require Protobuf transport and a separate
CanonicalV1 authority encoding. Delivery (D) Part I §6 requires one versioned
IDL, explicit compatibility rules and generated/validated language types.
These sources specify encoding properties, but do not assign wire field
numbers, a fixed endian, type tags, presence bytes or a hash-domain prefix.
The repository currently contains only an IDL placeholder.

AGENTS §2 restricts `core/contracts/` changes to contract-change work with an
ADR; §7 and CONTRACTS' modification rules require user approval. This ADR
proposes the initial encoding choices before publishing authoritative schemas.
It does not change the original specifications or their precedence.

## Proposed decision

1. Maintain Protobuf schemas as the sole message definition source under
   `core/contracts/`. Use explicit presence for singular scalar fields. Schema
   annotations identify required fields, ordered vectors, sets and set keys.
   Generate Rust/Python/TypeScript message types; never hand-copy message types.
2. Every message carries `schema_version`; initially only value 1 is accepted.
   Use the fully qualified Protobuf message name as its CanonicalV1 type tag.
   A source-mapped field table fixes field order to ascending field number.
   Do not recycle field numbers or enum discriminants. New authoritative
   fields/meaning require a new encoding version and migration.
3. Canonical message header: ASCII `NCV1`, followed by little-endian u32 schema
   version, then u32 byte length and strict UTF-8 bytes of the type tag.
   Body fields follow in fixed order; schema_version is carried in the header
   only. No protobuf wire bytes participate in the canonical body.
4. Integers use fixed-width little-endian encoding of their declared width;
   signed values use two's complement. Finite doubles use IEEE binary64 bits
   in little-endian order after normalizing negative zero to positive zero.
   Booleans are exactly one byte, 0 or 1; enum values are declared u32 codes.
5. Strings/bytes use u32 byte lengths. UTF-8 is strict: reject malformed bytes
   and lone surrogate inputs; preserve Unicode code points without implicit
   normalization. Optional fields use a 0/1 presence byte followed by the
   value when present. Missing and present-empty remain distinct. Required
   fields have no presence marker and must be supplied before encoding.
6. Collections use a u32 element count. Ordered vectors retain input order.
   Sets sort lexicographically by canonical bytes of the declared key;
   duplicate keys are rejected, never silently deduplicated. Nested messages
   use a u32 byte length followed by their complete canonical encoding.
   A oneof encodes its selected field number as u32, followed by that value;
   required oneofs reject absence. All trailing bytes are rejected.
7. Object digest = SHA256(ASCII `NEMU-CANONICALV1` + byte 0 + u32 type-tag
   byte length + UTF-8 type tag + canonical message bytes). The hash domain
   is unambiguous and separates different object types with identical bodies.
8. The transport decoder validates recursively before exposing a message:
   reject unknown schema, enum or field numbers, duplicate singular fields,
   multiple oneof selections, malformed lengths/UTF-8, nonfinite doubles,
   missing required fields, count overflow, and configured size/depth limits.
   It must not rely on protobuf's usual last-value-wins or unknown-field skip.
9. Rust is the authority encoder. Python integers and TypeScript bigint retain
   all 64-bit integer values. Input overflow is checked before narrowing.
   Deployment resource caps remain explicit versioned inputs; no deployment
   threshold is assigned by this ADR. u32 lengths are representation bounds,
   not permission to allocate that much memory.

An illustrative, **non-authoritative** Protobuf candidate is in
`reports/P5-01/initial-wire-candidate.proto`. This ADR approves only the
encoding policy and initial contract publication process. It does not approve
unspecified semantic fields, new governance rules, production thresholds or
missing Review_and_Remediation content. Such additions need their own review.

## Preserved decisions

- C01 remains unresolved; ADR 0006 remains PROPOSED; W09/W10 remain blocked.
- v1.0 remains NO-GROWTH. Defining/encoding an R/G object grants no runtime
  eligibility, authority, or growth capability.
- Frozen Architecture (F) §§7,7.1,8 supports qualification-branch and single-root
  ledger fields. Preserve these in source-mapped types only: branch, dwell
  start, content/support versions, revalidation records, impact metrics and
  cumulative single-root path; keep total-root consumption shared across
  branches. No runtime branch is installed.
- C02 is settled: all SYS01–SYS25 remain in the inventory.

## Alternatives

1. **Recommended:** approve this explicit encoding contract, then finish all
   schemas and cross-language golden vectors under P5-01.
2. Provide a different exact encoding policy before publishing the schemas.
3. Defer the wire contract; keep the independent validated numeric types,
   leaving P5-01 incomplete and its dependent cards blocked.

Hashing protobuf's serialized bytes or using generic JSON as authority are not
compliant alternatives under S §6 and D Part I §6.4.

## Consequences and acceptance

This makes implementation choices reviewable and gives all three languages one
byte contract. It creates no authority store or runtime route. The decoder and
golden corpus still need implementation after approval; this proposal is not
evidence of cross-language correctness.

Acceptance remains D Part I §10.8 and SYS21, with at least three legal and three
illegal vectors for **every** message, plus malformed UTF-8, duplicate fields,
unknown enum/schema/fields, set-order invariance, vector-order sensitivity,
negative zero, integer boundaries, size/depth rejection and type-separated
digests. Approval does not waive any test or permit automatic merge of an
incomplete P5-01.
