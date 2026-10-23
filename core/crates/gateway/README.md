# W18 gateway boundary components

This crate is **partial**, not a runnable authenticated gateway daemon. See
`tasks/requests/P5-03-gateway-interfaces.md` and `reports/P5-03/BLOCKED.md`.

- `framing::Framed`: a bounded u32 LE length prefix and generated Protobuf body.
  The caller supplies byte/item/depth and whole-frame time limits. Header and body
  share one deadline. Timeout, caller cancellation, malformed input and partial
  writes poison the session, preventing stream desynchronization on retry.
- `TerminalToken`: generates a per-installation 256-bit Windows CSPRNG secret,
  saves only current-user DPAPI ciphertext under a protected current-user DACL,
  reloads without replacing corrupt/existing tokens, and compares candidates with
  `subtle`. Secrets are zeroizing and have no Debug/Serialize implementation.
  No raw authentication wire format is invented here. Installation configuration
  owns the credential path; tests create credentials only in temporary directories.
- `AuthenticatedSession`: a process-local source capability minted after token
  verification. Device/session come from trusted installation configuration.
  It is not a deserializable peer claim. Current T1 permission decisions are
  supplied separately for each event, never cached in the authenticated session.
- `validate_event`: validates nine CONTRACTS 4.2 generated payloads and their
  canonical reference/digest bindings, ULID stable key, source identity, event kind,
  provenance type/derivation, result action ID and T1 fields. The local enum is
  never serialized. Strict generated parsing rejects unknown fields/enums,
  duplicates, unknown versions and invalid numeric fields.
- `ingest_validated`: prepares payload/provenance using **core-store**, verifies
  their references, then uses its durable ingress and quotas. It emits only the
  store's `RECEIVED_DURABLE`; this does not mean a core transaction committed.
  Object closure/lineage/durability checks remain W02's responsibility.
- Windows `pipe::Listener`: binds `\\.\pipe\nemu-core-<current SID>`, protected
  DACL with exactly the current SID, remote-client rejection, first-instance name
  protection and at most two kernel instances during rotation. Session processing
  is sequential through a mutable borrow. Disconnect creates the replacement with
  the same explicit ACL before closing the old handle, keeping name ownership
  continuous and preventing stale overlapped reads from crossing sessions.
  EXPERIMENT is rejected before creating a handle. There is no production bootstrap.

Only `windows.rs` permits unsafe code, with ownership/lifetime notes at each Win32
call. Named pipe and DPAPI behavior follow the [Tokio server API](https://docs.rs/tokio/1.54.1/tokio/net/windows/named_pipe/struct.ServerOptions.html),
[Microsoft pipe security](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights)
and [DPAPI](https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptprotectdata).
Same-account hostile code, an administrator taking ownership, or a compromised
trusted caller is outside this current-user ACL/bearer boundary.

Before calling `validate_event`, the trusted runtime must resolve provenance,
source/correction/parent references, allocate the authoritative D position and
ingest time, and bind action-result authority. Those peer claims are not certified
by this component alone. The public store adapter does not perform those missing
steps or implement the growth/runtime pipeline.

Checks (Windows, D-drive temporary roots/caches):

```powershell
& ./reports/P5-03/check.ps1
```

The user has supplied a successful real second-account denial result on Windows
10 (Win32 error 5, exit 0); the original user message and evidence provenance are
saved under `reports/P5-03/second-account/`. Full window transcripts were not
attached. This evidence is separate from the automated same-account tests.
The suite never opens user stores.

Windows SYS01 structural tests connect real pipes to public core-store APIs:
durable receive, delayed/interleaved commit receipts, retransmission/conflict with
unchanged H/state/anchor, and uncommitted close/reopen. A test-only coordinator
preloads synthetic payload/provenance and authenticates locally; it is not a wire
authentication/upload protocol or a full runtime. Close/reopen is not power loss.
