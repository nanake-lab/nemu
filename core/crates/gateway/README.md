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
- `AuthenticatedSession`: a process-local source/grant capability minted after
  token verification. Device/session/grants must come from trusted installation
  and permission configuration. It is not a deserializable peer claim.
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
  protection and one instance. It retains that handle across disconnect/reconnect;
  dropping/rebinding at every session would open a pipe-name ownership gap.
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

Second-account denial still requires a real second Windows account arranged by
the user. An exact live-handle DACL inspection is supporting evidence, not a
substitute for that acceptance test. The suite never opens user stores.
