# W19 fact view

`FactView<'a, L>` borrows `core_store::Reader<L>`. It opens no authority database or production path. `rebuild(reader, commit, limit, embedder)` obtains a verified complete committed transaction prefix; a fresh private in-memory SQLite index is built from that prefix. Index rebuilding does not change H/x/R/theta/L/Q. A borrowed Reader and the lineage type prevent cross-lineage calls.

`FactQuery` / `Budget` / `Access` / `RetrievalTrace` are in-process Rust plans, not IPC messages. IPC adapters use generated IDL `Query`, `QueryResult`, `FactProjection`, `FactItem` and `UserRetraction`. No IDL or existing bytes are changed. `user_view` currently serves `MEMORY`; actions/run_history/growth remain their owning services. The P5-22 successor adapter is aligned only after #57 reaches main.

## Query and permissions

- Explicit commit, fixed query version, event/payload type, device source, inclusive signed-ns time bounds, text/current observation, accepted entity identities, item/byte/step budgets and purpose.
- Panel filters accept decimal ns, RFC3339 offsets, or YYYY-MM-DD (UTC start/end of day). Known payload names such as user_text are supported as well as EventKind names. Invalid dates/kinds fail.
- `Access::default()` denies. An empty source set with allowed=true is an explicit unrestricted local grant; a nonempty source set restricts every channel before RRF. The caller must obtain the grant from its permission layer. No worker or panel supplies an authorized=true field across IPC.
- Default caps: c_core 16 items/16 KiB/100k steps; c_task 32/32 KiB/200k; user_view/audit 50/64 KiB/300k. Caller/versioned deployment resources may choose stricter caps. Steps charge deterministic Rust work and SQLite VM opcodes (progress handler); the pinned model has 512-token truncation and four CPU threads. These are retrieval cost units, not a wall-clock deadline. Bytes include projection encoding and original payloads; panel responses also enforce their encoded size.
- SQL literal phrase matching never executes user-supplied FTS operators. Trigram handles >=3 Unicode characters; shorter literals use a bounded scan. Exact event/type/time/source filters always apply.
- Entity identities come only from ACCEPTED SemanticGraph Ref records associated with that event. Accepted source-span mentions must be present in the original text before they can match o and recall the same identity. Unaccepted graphs, appraisals, match scores, candidates and annotations are not inputs.
- Time recall runs for an explicit interval or an empty-text listing. sqlite-vec computes cosine distances on authorized/effective documents. RRF uses fixed k=60, top 64/channel, fixed entity/time/FTS/vector order, and source-ID tie breaks. No learned reranker or abstractive paragraph summarizer is enabled. Raw paragraphs remain the authority; future summaries must retain H provenance and be rebuildable.
- Each returned item re-reads both its EventEnvelope and payload through Reader, checking original identity/type/hash/length/text. Missing objects return OBJECT_MISSING. Unknown/incomplete prefixes fail rebuilding. A changed current transaction returns INDEX_LAGGING before task/memory retrieval; an old index cannot serve a newly withdrawn fact.
- Withdrawal controls are never indexed as fact text, including notes. Withdrawals suppress c_task and MEMORY, even when include_retracted=true. c_core retains the experience as evidence; explicit audit retains originals. Replacement/correction chains are applied, including withdrawals via event/object/accepted-record references and predecessor/successor chains.

## Audit and integration boundary

`Execution` contains a generated projection and its complete actual query/permission/model/vector/rank/result trace. `trace.audit_record(&projection)` creates a generated AuditExtension object containing versioned audit data, never a fact event. The outer trusted runtime must stage/pin this object and record its reference in that event transaction's audit_refs; user_view must likewise retain it in its read-only audit channel. This crate owns no CommitCapability and cannot publish it itself. Do not turn the returned audit object into a staged_h payload. Structural replay consumes recorded projections/results and never calls the embedder again. Runtime orchestration is P5-08/P5-19/P5-29, not this card.

## Pinned CPU model

See models/manifest.json and ADR 0020. `cpu-embedding` uses offline, hash-verified official intfloat/multilingual-e5-small, revision 614241f622f53c4eeff9890bdc4f31cfecc418b3, MIT, 384 dimensions, masked mean/L2 pooling, query:/passage: prefixes, 512 tokens and CPU 4 threads. `CpuEmbedder::load` selects FP32. The manifest retains a rejected official quantized candidate's provenance and SHA-256; it failed the fixed positive controls, has no runtime loader and is skipped by the download script. No CUDA/DirectML feature is enabled, no runtime model download or Python subprocess exists. Optional feature compilation fetches ORT CPU build artifacts, so set ORT_CACHE_DIR as well as CARGO_HOME/CARGO_TARGET_DIR before building.

```powershell
$env:CARGO_HOME='D:\Dev\cache\cargo'
$env:CARGO_TARGET_DIR='D:\Dev\cache\cargo-target\P5-04'
$env:ORT_CACHE_DIR='D:\Dev\cache\ort'
$env:TEMP='D:\Dev\cache\tmp'
$env:TMP=$env:TEMP
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
