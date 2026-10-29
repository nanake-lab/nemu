# 2026-10-24 core-store V3 轻量核验

工作区 `D:\Dev\nemu-wt\P5-34`；基线 main `0b5412e95caab447c70148c473e544dbab28c57c`。
routine_reporter 按角色名执行机械命令，主代理检查返回结果；这不是交叉审查。

## SQL 探针

命令 `python reports/P5-34/verification/core-store-v3/check_store_sql.py`；退出码0。
时间：2026-10-24 00:51:59.8362859–00:51:59.9258716 +08:00。

```text
SQLite 3.49.1; 7 SQL probes passed using source-extracted queries
Scope: latest-before-association, empty successor, reassignment, limit+1, complete accepted ordering
Not run: Rust compilation, core-store tests, G replay, SYS17 or crash acceptance
```

## 空白与 Rust 语法/格式

- `git diff --check`；退出码0，无输出。时间00:52:02.7234442–00:52:02.7763788 +08:00。
- 在 core/ 执行 `cargo fmt -p core-store -- --check`；退出码0，无输出。时间00:52:07.8057787–00:52:08.0902796 +08:00。仅该进程设置 `CARGO_TARGET_DIR=D:\Dev\cache\cargo-target\P5-34`。

Rustfmt 不检查类型，不编译，不运行测试。上述命令均未启动 Rust 编译、core-store 测试、完整三语言检查或 G 重放。

## 静态入口核对

`rg -n 'stage_with_replay|rebuild\(&input\)|actual_retention|evidence_absolute' core/crates/core-store/src --glob '*.rs'`；退出码0。
核验时 `r_replay.rs:182` 显式调用 `rebuild(&input)`，`:204` 才进入 `stage_with_replay(..., true)`；`transaction.rs:113` 普通入口传 false。q/V 的数值赋值只在 tests/ 合成夹具和负例中，生产存储路径没有默认 q=1 或从 B−c 转换 V。

以上是源代码检查；编译、所有存储测试、真实 G 迁移集成及 SYS17 均为 not_run。
