# P0-06 build baseline — 2026-10-14

Base main: `ab3c6d175f0c4ea156aa9a8baff429e741d3eb49` (P0-02 merged). This is the measured Windows baseline, not a promise that hosted CI has the same OS/compiler. Existing manifests and lockfiles remain authoritative; no production dependency is added by this card.

| Component | Measured / selected baseline | Pin and reproduction |
| --- | --- | --- |
| Rust | 1.99.0, b940084d7eb6a299eb4bfeb8e34901bc051e7ac4, LLVM 23.1.1 | core/rust-toolchain.toml; `rustc +1.99.0 -Vv` |
| Target | x86_64-pc-windows-msvc, little endian, 64 bit | `rustc +1.99.0 --print cfg`; default target, no native-CPU flags |
| Enabled target features | cmpxchg16b, fxsr, sse, sse2, sse3 | Compiler cfg evidence; not a hardware capability inventory |
| Node / pnpm | v24.21.0 / 12.11.2 | .nvmrc selects 24; packageManager and pnpm-lock.yaml pin package tooling/dependencies |
| Python / uv | 3.12.14 / 0.12.15 | Four project uv.lock files; `uv run --project workers/irl python --version` |
| .NET SDK | 10.0.401 | executors/office-connector/global.json permits latestPatch; future patch changes require re-recording |
| Windows | Windows 10 Home Chinese, 10.0.19045, build 19045, 64 bit | Get-CimInstance Win32_OperatingSystem, non-identifying fields only |
| SQLite probe | rusqlite =0.40.2, bundled; libsqlite3-sys 0.38.2; SQLite **3.53.2** | reports/P0-06/sqlite-probe/Cargo.lock; run with --locked |
| SQLite C compiler | msvc-1944 | Actual PRAGMA compile_options in sqlite-build.log |

No RUSTFLAGS, CARGO_ENCODED_RUSTFLAGS or CARGO_BUILD_TARGET override was present during the probe. Authoritative numerical code must use finite binary64, deterministic summation and no fast-math (S §6); this report does not infer numeric reproducibility from a compiler version alone. CPU-specific optimization changes require a refreshed fingerprint and golden-vector/replay validation. No model, genesis profile, threshold or deployment calibration is fabricated; S §31 assigns these to their later tasks.

## Bundled SQLite and WAL

Actual SQL fingerprint: `2026-06-03 19:12:13 d6e03d8c777cfa2d35e3b60d8ec3e0187f3e9f99d8e2ee9cac695fd6fcdf1a24`.

SQLite's [WAL documentation §11](https://www.sqlite.org/wal.html) states that the WAL-reset corruption bug affects 3.7.0 through 3.51.2 and is fixed in 3.51.3 and later. The [3.53.3 release history](https://www.sqlite.org/releaselog/3_53_3.html) also lists the fix among prior 3.53.0 changes. The measured bundled 3.53.2 includes that fix; do not confuse the crate version with SQLite's version. [rusqlite's bundled feature](https://github.com/rusqlite/rusqlite) builds the embedded SQLite rather than relying on an arbitrary system DLL.

The isolated probe verifies WAL selection, synchronous=FULL, rollback, integrity_check and cleanup in a newly created temporary directory. It does **not** prove concurrent checkpoint race resistance, power-loss durability, crash recovery, backup correctness or SYS17. Those remain P5-02/P5-15 work. The app's core-store still has no rusqlite dependency: this exact locked selection is recorded for that task to adopt and validate, not secretly installed into production.

```powershell
cargo +1.99.0 run --locked --manifest-path reports/P0-06/sqlite-probe/Cargo.toml
cargo +1.99.0 clippy --locked --manifest-path reports/P0-06/sqlite-probe/Cargo.toml -- -D warnings
pwsh -File reports/P0-06/verify-baseline.ps1
```

Evidence: `reports/P0-06/sqlite-build.log`, timestamped verification directory (environment, source hashes, dependency licenses and lockfile hashes). rusqlite is MIT; libsqlite3-sys is MIT; upstream SQLite is public domain. These are report-probe dependencies only. The lockfile pins transitive versions/checksums; no GPL/AGPL application dependency is introduced.
