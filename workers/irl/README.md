# P5-07 IRL candidate foundation

This is a process-local, offline-testable library. It has no production entry point,
IPC listener, model launcher, authority storage handle, retry loop or background queue.
P5-07 remains partially delivered. See [the report](../../reports/P5-07/REPORT.md).

`interpret` consumes an asynchronous local inference stream and returns a
`RecordedCandidate`: actual bytes in the generated `AuditExtension`, bound P0-08 wire
candidate, producer/version, a copied `ProposalBindingV2`, and completion/error
information. Failed or cancelled output is retained; oversized output retains only
the exact bounded byte prefix and is marked incomplete. No raw exception is logged.
Callers own the returned record's lifetime; the library does not persist it.

ADR 0031 §4 selects the original wire schema. The packaged JSON is byte-identical to
`spikes/codex-app-server/semantic-candidate.wire.schema.json`, verified by a test.
It still has the original `p0-08-NNNN` source-ID pattern. `p0-08-0000` is a per-request
model alias, **not** a new event identity: the original identity stays in the binding.
Each model node copies its literal excerpt after the fixed `p0-08-0000#` prefix
in its single `sourceRefs` entry and emits the constant `[0, 0]` for `sourceSpan`.
The worker searches only field 2 of the bound UserText or UserUtterance.
Exactly one occurrence yields a generated G01 SourceLocation with a RecordVersion
and a half-open UTF-8 byte range. The parsed P0-08 node receives its plain alias
and computed code-point range. The audit retains the original model bytes.
An empty, absent or repeated excerpt leaves the candidate unresolved. The
`sources` map retains each node's location, `no_match` or `ambiguous` result.
Facts, audio and speaker identity are outside this search.

The injected async generator is the minimal offline test seam, not a backend registry
or a real endpoint adapter. It must yield cooperatively and close on cancellation.
Real local m=0, constrained decoding, fact/registry object access, remote bridge use,
formal SemanticCandidateV2 mapping and authenticated transport await the owning cards.
The function can run while a reply completes, but the production reply scheduling and
late-result commit policy are not implemented here. There is no default deadline;
an explicit caller deadline, when supplied, is enforced without retry.

Checks (from this directory): `uv run ruff check .`, `uv run mypy --strict src tests`,
`uv run pytest`, `uv build`. Tests use synthetic inputs and fixed golden DTOs, never
user storage or model inference. They do not measure semantic accuracy or prove SYS02
end to end. jsonschema 4.26.0 is the same version used in P0-08 (MIT); its locked
transitive dependencies are listed in the report's verification metadata.
