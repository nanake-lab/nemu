# P5-04: core-store committed-prefix read API

Scope request: append a read-only Reader::transactions_at(commit: &str, limit: usize) -> Result<Vec<CoreTransactionV2>> in core/crates/core-store/src/queries.rs, and temporary-store tests in core-store/src/tests/reader.rs.

W19 must rebuild entity recall only from ACCEPTED SemanticGraph references in each committed CoreTransactionV2.accepted_results. Reader::facts currently returns only EventEnvelope; Reader::current loses prior accepted-result associations. Directly opening the authority SQLite database from fact-view would bypass core-store.

Required semantics: explicit known commit; ordered complete prefix, bounded and reject rather than truncate on insufficient budget; generated IDL messages; no write API, schema/format change, or new production-path access. Existing Reader::read remains the object visibility/integrity boundary.

P5-04 implementation can continue independently, but full store-backed rebuilding requires this small API addition. Awaiting user authorization for the two paths outside the task card.
