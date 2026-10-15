# ADR 0007 — Initial IDL and CanonicalV1 encoding contract

Status: **APPROVED WITH AMENDMENTS — user approval recorded 2026-10-14**.
Date: 2026-10-14. Task: P5-01 / W01. Change class: **契约变更**.

## Background

System Plan (S) §§2,6 and P5-01 require Protobuf transport and a separate
CanonicalV1 authority encoding. Delivery (D) Part I §6 requires one versioned
IDL, explicit compatibility rules and generated/validated language types.
These sources specify encoding properties, but do not assign wire field
numbers, a fixed endian, type tags, presence bytes or a hash-domain prefix.
At proposal time the repository contained only an IDL placeholder.

AGENTS §2 restricts `core/contracts/` changes to contract-change work with an
ADR; §7 and CONTRACTS' modification rules require user approval. This ADR
records the approved initial encoding choices for authoritative schemas.
It does not change the original specifications or their precedence.

## Approved decision (including the user's amendments)

1. Maintain Protobuf schemas as the sole message definition source under
   `core/contracts/`. Use explicit presence for singular scalar fields. Schema
   annotations identify required fields, ordered vectors, sets and set keys.
   Generate Rust/Python/TypeScript message types; never hand-copy message types.
2. Every message carries `schema_version`; initially only value 1 is accepted.
   Use an independently assigned, frozen UTF-8 string as its CanonicalV1 type
   tag. Store the tag explicitly in the schema registry; never derive it from
   a Protobuf message name, package, language symbol or file path. Renaming a
   Protobuf symbol preserves the tag. Changing the tag is a new encoding
   version, with explicit migration; tags are never recycled.
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
10. Extensible audit fields have their own explicit schema/version, separate
    from the authority message schema (S §6). Audit payload is not an unknown
    field escape hatch in authority messages. It is retained as a separately
    versioned audit object or reference, with its actual bytes recoverable.
    Audit-only additions cannot participate in undeclared causal computation.
    Promoting any audit datum into an authority input requires a declared
    authority schema/version and the normal migration/governance process.
11. Freeze point: before N3 birth, encoding may be revised only after a backup
    and an explicit migration description (including recovery/rollback and
    treatment of existing bytes/digests). Old records must never be silently
    reinterpreted. After the genesis manifest is signed, its encoding contract
    is locked; every later change requires a new encoding version and migration.
    Changing a type tag always counts as a new encoding version. Tests use
    temporary storage and never open the user's rehearsal or life storage.

## Hand-calculated hexadecimal example

This example is derived field by field, not generated by the encoder. Take
OutputEnd with frozen independent tag `echo.output-end`, schema_version = 1,
required output_id = `A`, and required reason = DONE (enum code 1). The two
body fields have no optional presence byte because they are required.

| Part | Bytes (hex) | Length |
| --- | --- | --- |
| NCV1 header magic | `4e 43 56 31` | 4 |
| schema version, u32 LE | `01 00 00 00` | 4 |
| tag byte length = 15, u32 LE | `0f 00 00 00` | 4 |
| UTF-8 `echo.output-end` | `65 63 68 6f 2e 6f 75 74 70 75 74 2d 65 6e 64` | 15 |
| output_id length = 1, u32 LE | `01 00 00 00` | 4 |
| UTF-8 `A` | `41` | 1 |
| DONE = 1, u32 LE | `01 00 00 00` | 4 |

The complete canonical message is 36 bytes:

```text
4e435631010000000f0000006563686f2e6f75747075742d656e64010000004101000000
```

The SHA-256 input **includes the complete NCV1 header**. Domain prefix plus
delimiter is 17 bytes, tag length and tag are 19 bytes, and message is 36
bytes: 72 bytes total. The exact input is the concatenation of these lines:

```text
4e454d552d43414e4f4e4943414c563100
0f0000006563686f2e6f75747075742d656e64
4e435631010000000f0000006563686f2e6f75747075742d656e64010000004101000000
```

Nested messages **repeat their own NCV1 header**, schema version and frozen
type tag. If this OutputEnd is a nested value, its representation is u32 LE
length `24 00 00 00` followed by the complete 36 bytes above (40 bytes total).
Its header is not removed merely because the enclosing message also has one.
Nested encoding does not add a nested hash-domain prefix; the outer digest
hashes the outer canonical bytes, including the nested header and body.

This example is an additional independent golden oracle, not a replacement
for the required three legal and three illegal vectors for every message.

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

1. **Selected:** approve this explicit encoding contract, then finish all
   schemas and cross-language golden vectors under P5-01.
2. Provide a different exact encoding policy before publishing the schemas.
3. Defer the wire contract; keep the independent validated numeric types,
   leaving P5-01 incomplete and its dependent cards blocked.

Hashing protobuf's serialized bytes or using generic JSON as authority are not
compliant alternatives under S §6 and D Part I §6.4.

## Consequences and acceptance

This makes implementation choices reviewable and gives all three languages one
byte contract. It creates no authority store or runtime route. The decoder and
golden corpus are implemented in P5-01. Approval itself is not evidence of
cross-language correctness; the measured results are in reports/P5-01/acceptance/.

Acceptance remains D Part I §10.8 and SYS21, with at least three legal and three
illegal vectors for **every** message, plus malformed UTF-8, duplicate fields,
unknown enum/schema/fields, set-order invariance, vector-order sensitivity,
negative zero, integer boundaries, size/depth rejection and type-separated
digests. Approval does not waive any test or permit automatic merge of an
incomplete P5-01.
