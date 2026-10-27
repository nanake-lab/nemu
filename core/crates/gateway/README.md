# W18 Windows gateway

`gateway::Gateway::serve` is the authenticated, bounded named-pipe service. Its
owner supplies the sole `core_store::Store`, trusted installation IDs/DPAPI tokens,
deployment budgets, and a `hooks::CoreHandler`. The gateway creates no database,
protocol types, business permission store, or growth operators.

- The pipe is `\\.\pipe\nemu-core-<current SID>`, with a protected current-SID-only
  DACL, remote rejection and first-instance protection. A successor with the same
  explicit ACL is created before handing off each connected handle. Up to 14
  concurrent clients share one serial Store actor; the 16-instance kernel ceiling
  leaves space for rotation. EXPERIMENT is rejected before binding.
- Pre-authentication framing uses static deployment byte/item/depth/deadline
  limits. Only the generated `TerminalAuthenticate` is accepted first. DPAPI
  protects the 256-bit per-installation token at rest; token buffers are zeroizing,
  comparison uses `subtle`, and no credential is logged or persisted in core-store.
  Every connection gets a fresh random session with server-owned identity,
  capabilities and budget. A session ID cannot authenticate another connection.
- Generated `TerminalRequest`/`TerminalResponse` and `ReceiptV2` implement all six
  ingress kinds. The core-types pairing/session/request/response checks are used
  at the boundary. Superseded messages remain historically decodable but are
  rejected on live frames. No local enum is serialized.
- Uploads reserve the full declared canonical object length before the first
  chunk, enforce sequential offsets/stable declarations, verify CanonicalV1 and
  the domain-separated digest, and call the existing typed Store preparation API
  before COMPLETE. `build.rs` generates dispatch to existing IDL-generated Rust
  types from the official schema, never wire types. Transient, superseded and
  ingress-envelope objects cannot be uploaded. Completed objects retain quota and
  Store pins until consumed, expired or disconnected; Store GC owns reclamation.
- Events resolve the nine CONTRACTS 4.2 payload types and provenance, validate
  their semantic/reference bindings, and obtain current T1/source/action
  authority from `CoreHandler::authorize_event` for every event. Only
  `Store::ingest_terminal` assigns D position/time and returns RECEIVED_DURABLE.
  Gamma runs before each core transition. Only accepted, lineage-visible
  committed transactions can produce CORE_COMMITTED; delayed/reordered results
  retain their original request IDs. Publication is observed after each serial
  core step and retained only in the bounded pending-request correlation map.
- Query results get contextual correlation/content-isolation validation. Approval,
  cancel and growth reach trusted handlers and get ACK only on handler success;
  defaults deny. The approval handler must resolve the actual outstanding request
  and execution bytes and use `terminal::validate_approval`/its one-use snapshot.
  Authentication and a structurally valid decision do not confer execution or
  growth-management authority. Runtime/action-gate integration belongs to its
  owning cards; SYS01 uses a synthetic actual Store transaction handler.
- Rejections use core-store's independent bounded durable enum audit. No client
  text/token enters it. Typed Store quotas and bounded in-memory correlation return
  safe capacity reasons before D reception. An exhausted audit stops intake;
  audit/storage/witness errors terminate service without inventing success or a
  safe rejection. A full output queue disconnects: the peer retries the same event
  ID/stable key after an unknown outcome. There is no second receipt journal.

Session/upload lifetimes, connection count, parser limits, upload reservations,
pending request metadata and output queues all have explicit bounds. Session and
upload expiry are checked on admission as well as by the timer. Same-account
hostile code, administrator ownership changes and compromised trusted handlers
are outside the current-SID/bearer boundary.

Run on Windows with D-drive caches and temporary roots:

```powershell
& ./reports/P5-03/post-P5-35/check.ps1
```

Native tests cover DPAPI reload, authentication/replay, uploads/expiry/disconnect/
GC, six-way dispatch, per-event revocation, durable rejection/audit failure,
actual Store record/byte backpressure, concurrent pipe ACLs, and SYS01 with real
Store commits. Fresh gateway child processes test SQL-before/SQL-after crashes
and TerminateProcess after durable reception. All data is synthetic in freshly
owned temporary directories; the child process's LOCALAPPDATA points only
inside its parent-owned sandbox. No user store or system setting is accessed.
These are process-death/recovery tests, not a physical power-cut measurement.

The real Windows 10 second-account denial result is user-provided and retained
verbatim under `reports/P5-03/second-account/`. Full terminal-window transcripts
were not supplied. Current automated tests verify the actual DACL on concurrent
and successor handles; they do not claim a new second-account run.
