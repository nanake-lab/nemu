# W19 fact view

`FactView<'a, L>` borrows `core_store::Reader<L>`. It opens no authority database or production path. `rebuild(reader, commit, limit, embedder)` obtains a verified complete committed transaction prefix; a fresh private in-memory SQLite index is built from that prefix. Index rebuilding does not change H/x/R/theta/L/Q. A borrowed Reader and the lineage type prevent cross-lineage calls.

`FactQuery` / `Budget` / `Access` / `RetrievalTrace` / `PageCursor` / `MemoryEntry` are in-process Rust plans, not IPC messages. IPC adapters use generated IDL `QueryV2`, `QueryResultV2`, `FactProjection`, `FactItem` and `UserRetraction`, aligned with merged P5-22 (#57). No IDL or existing bytes are changed. `user_view` serves the first `QUERY_MEMORY` page and always returns empty growth_entries; actions/run_history/growth remain their owning services and are denied here. Trusted callers use `user_view_page` and retain `Execution.next_page` separately from the unchanged terminal message. Terminal cursor transport belongs to its contract/integration card; no cursor is hidden in query text/date fields.

## Query and permissions

- Explicit commit, fixed query version, event/payload type, device source, inclusive signed-ns time bounds, text/current observation, accepted entity identities, item/byte/step budgets and purpose.
- Panel filters accept decimal ns, RFC3339 offsets, or YYYY-MM-DD (UTC start/end of day). Known payload names such as user_text are supported as well as EventKind names. Invalid dates/kinds fail.
- `Access::default()` denies. An empty source set with allowed=true is an explicit unrestricted local grant; a nonempty source set restricts every channel before RRF. The caller must obtain the grant from its permission layer. No worker or panel supplies an authorized=true field across IPC.
- Default caps: c_core 16 items/16 KiB/100k steps; c_task 32/32 KiB/200k; user_view/audit 50/64 KiB/300k. Caller/versioned deployment resources may choose stricter caps. Steps charge deterministic Rust work and SQLite VM opcodes (progress handler); the pinned model has 512-token truncation and four CPU threads. These are retrieval cost units, not a wall-clock deadline. Bytes include projection encoding and original payloads; panel responses also enforce their entire canonical encoded size, including the echoed query_id. If even the empty response envelope cannot fit, user_view/user_view_page return Error::Invalid before retrieval rather than an oversized QueryResultV2. IDs are never shortened. At the exact envelope limit an empty result can retain its NO_MATCH/DENIED/INDEX_LAGGING status; trimming all matched items returns a bounded BUDGET_EXHAUSTED result without a continuation.
- SQL literal phrase matching never executes user-supplied FTS operators. Trigram handles >=3 Unicode characters; shorter literals use a bounded scan. Exact event/type/time/source filters always apply.
- Entity identities come only from ACCEPTED SemanticGraph Ref records associated with that event. Accepted source-span mentions must be present in the original text before they can match o and recall the same identity. Interpretation content never creates entity identities.
- V2-88: c_task/MEMORY can recall the six accepted appraisal/match/candidate/annotation types. They keep their typed ObjectRef, source event ID, exact event time, and separate backfill recording time. `Execution.memories()` exposes task/user/audit semantic fields without binding/state references, marked `这是她当时的反应`; `MemoryEntry::marked_text()` and panel summaries preserve the marker. They are historical interpretations, not facts or new evidence. Unaccepted records, arbitrary reference closures, state and ledger are never traversed. c_core filters reactions before every recall route, uses a fact-only FTS corpus and embedding batch, and returns its generated FactProjection without task presentation data. Reactions inherit their source experience's vector; their own semantic fields remain directly searchable by FTS, avoiding generic record metadata outranking the original experience.
- Time recall runs for an explicit interval or an empty-text listing. sqlite-vec computes cosine distances on authorized/effective documents. Query plan `w19-rrf-v2` uses fixed k=60, entity/time/FTS/vector order and stable record-ID tie breaks. c_core keeps its first-round top 64/channel; memory pages rank the entire matching range without a total-history/top-64 cutoff. Item/byte/step budgets bound each call. No learned reranker or abstractive paragraph summarizer is enabled. Raw paragraphs remain the authority.
- V2-76: copy `Execution.next_page` into `FactQuery.after` to continue. The opaque cursor binds prefix, query, filters, purpose, authorization, budgets and model/index versions. Rebuilding an identical index preserves continuation; there is no session state. A partial page returns BUDGET_EXHAUSTED plus a cursor when it made progress; zero-progress budget results have no cursor. Exact text, decimal nanoseconds and equal-time ordering survive all pages. New commits yield INDEX_LAGGING during paging too, preventing withdrawal leaks. NO_MATCH, DENIED and INDEX_LAGGING remain distinct alongside OBJECT_MISSING and BUDGET_EXHAUSTED.
- Each returned item re-reads both its EventEnvelope and payload through Reader, checking original identity/type/hash/length/text. Missing objects return OBJECT_MISSING. Unknown/incomplete prefixes fail rebuilding. A changed current transaction returns INDEX_LAGGING before task/memory retrieval; an old index cannot serve a newly withdrawn fact.
- Withdrawal controls are never indexed as fact text, including notes. Withdrawals suppress the original and associated reactions in c_task and MEMORY even when include_retracted=true. c_core retains the experience as evidence; audit retains originals. Replacement/correction chains apply, including event/object/accepted-record references and predecessor/successor chains. Backfilled reactions require access to both original and recording sources.

## Audit and integration boundary

`Execution` contains a generated projection and its complete actual query/permission/model/vector/rank/result trace. `trace.audit_record(&projection)` creates a generated AuditExtension object containing versioned audit data, never a fact event. The outer trusted runtime must stage/pin this object and record its reference in that event transaction's audit_refs; user_view must likewise retain it in its read-only audit channel. This crate owns no CommitCapability and cannot publish it itself. Do not turn the returned audit object into a staged_h payload. Structural replay consumes recorded projections/results and never calls the embedder again. Runtime orchestration is P5-08/P5-19/P5-29, not this card.

## Pinned CPU model

See models/manifest.json and ADR 0020. `cpu-embedding` uses offline, hash-verified official intfloat/multilingual-e5-small, revision 614241f622f53c4eeff9890bdc4f31cfecc418b3, MIT, 384 dimensions, masked mean/L2 pooling, query:/passage: prefixes, 512 tokens and CPU 4 threads. `CpuEmbedder::load` selects FP32. The manifest retains a rejected official quantized candidate's provenance and SHA-256; it failed the fixed positive controls, has no runtime loader and is skipped by the download script. No CUDA/DirectML feature is enabled, no runtime model download or Python subprocess exists. Optional feature compilation fetches ORT CPU build artifacts, so set ORT_CACHE_DIR as well as CARGO_HOME/CARGO_TARGET_DIR before building.

```powershell
$env:CARGO_HOME='D:\Dev\cache\cargo'
$env:CARGO_TARGET_DIR='D:\Dev\cache\cargo-target\P5-04'
$env:ORT_CACHE_DIR='D:\Dev\cache\ort'
# Keep inherited TEMP/TMP; Windows CI uses RUNNER_TEMP. Do not change ACLs.
$env:HF_HOME='D:\Dev\cache\huggingface'
./core/crates/fact-view/scripts/download-model.ps1
$env:NEMU_W19_MODEL='D:\Dev\models\multilingual-e5-small\614241f622f53c4eeff9890bdc4f31cfecc418b3'
cd core
cargo test --workspace --locked -- --test-threads=1
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy -p fact-view --all-targets --features cpu-embedding --locked -- -D warnings
cargo test -p fact-view --release --features cpu-embedding real_cpu_multilingual --locked -- --ignored --nocapture --test-threads=1
```

The explicit real-model test uses only a new TemporaryRoot EXPERIMENT store, restarts it, measures a small three-language recall skeleton and runs 100 vector-enabled withdrawal queries. Ordinary CI does not download model weights. The full multi-year/load/full-answer acceptance (completion criterion #15) remains P6-13; the small skeleton does not certify 90% full-answer accuracy or a 50 ms workload SLO.
