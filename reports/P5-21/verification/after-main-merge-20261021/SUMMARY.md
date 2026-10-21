# 合并 main 之后的复跑（2026-10-21）

合并 origin/main（ab1acb5）之后，在 Claude 云端会话中复跑。工具链说明见 toolchain.txt：rust-toolchain.toml 锁定的 1.99.0 下载被代理拒绝，按 AGENTS.md 第 10 节用环境内的 stable；以 CI 的结果为准。

| 检查 | 结果 | 日志 |
| --- | --- | --- |
| `generate.py --check` | 通过，0 个生成文件过期 | generate-check.log |
| `verify.py`（`pnpm test:core-contracts` 的内容） | 通过：Rust、Python、TypeScript 各 9,004 例；官方 protoc 互操作 378 条；对 p5-01 基线 0 违例 | core-contracts.log |
| `cargo fmt --check`、`clippy -D warnings`、`cargo test --workspace --locked` | 通过，46 个测试，0 失败 | rust-*.log |
| `pnpm lint`、`typecheck`、`test`、`build`、`test:contracts` | 通过，vitest 443 个测试 | pnpm-*.log |
| 契约工具 ruff、mypy --strict（含 CI 中本卡追加的一步） | 通过 | contract-*.log |
| 四个 Python 项目 ruff、mypy --strict、pytest | 通过 | workers-*、services-* |

本地直接运行 verify.py，并在 PATH 前加了一个 pnpm 包装脚本：这个沙箱里，`pnpm run` 加进 PATH 的 pnpm 是一个不可执行的占位文件。CI 不受影响。
