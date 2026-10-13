P0-01 establishes pnpm, Cargo, uv and .NET workspaces with buildable skeletons and an Electron application that opens a transparent stage and a panel. It adds Ubuntu static/unit CI and Windows transport, integrity, Electron and visual checks.

The legacy migration adapts only the controlled Composer UI. No mock agent, legacy state store, credentials, character assets, or business logic was imported. Unimplemented commands fail explicitly with their owning task.

Validation: all 34 local checks pass (TypeScript lint/typecheck, 13 unit tests, builds, 4 Python tests plus strict checks, Rust fmt/clippy/test, .NET build, transport, Low IL, Electron and visual smoke). Rust crates are empty skeletons; no Echo Core behavioral acceptance is claimed. The Electron smoke additionally passed three consecutive runs after fixing temporary-profile cleanup and render readiness.

Report: [reports/P0-01/REPORT.md](reports/P0-01/REPORT.md)
Evidence: [reports/P0-01/final-validation/](reports/P0-01/final-validation/)
Migration: [reports/P0-01/migration.md](reports/P0-01/migration.md)
Setup: [docs/SETUP.md](docs/SETUP.md)

Remote CI is being checked; final run links and any blockers will be appended to the report. No architecture deviation or contract change is proposed.

Final implementation validation: [CI 38440863374](https://github.com/nanake-lab/nemu/actions/runs/38440863374) passed both Ubuntu and Windows jobs on `f0e3442`, including native-alpha Electron smoke, named pipes, Low IL and exact visual comparison. The initial Windows SDK permission failure was fixed; cancelled runs are not counted as passes. No implementation blockers remain. Final evidence is appended to REPORT.md; documentation HEAD CI is checked before any repository-authorized merge.
