# P5-43 基线及黄金向量待 P5-40

P5-40 与 P5-43 都改 `core/contracts/`。[#235](https://github.com/nanake-lab/nemu/pull/235) 仍为 OPEN，head 为 `59a79fab`。其报告记录 W18 等待网关修复；[修复 #240](https://github.com/nanake-lab/nemu/pull/240) 的状态由该卡维护。

本卡已进行了独立的 IDL 生成、兼容检查、Rust 语义检查与三端上传检查。生成物供编译和审查候选字段使用。正式冻结和追加式黄金向量按派卡顺序等待 #235。`pnpm test:core-contracts` 已实跑，`verify.py` 在每个新消息至少 3 条合法向量的断言返回 1。22 个新标签当前各有 0 正、0 反，SYS21 缺少最低向量覆盖，三语言新语义对照、生产存储和整卡验收尚未完成。

1. 现在可交叉审查 ADR、字段及反例，发现问题可在本分支修订草案。
2. #235 合入后在此分支 merge 最新 `origin/main`，由生成器重建文件；从当时 main 的 IDL 和旧向量冻结基线，再追加并实跑新向量、完整检查和交叉复审。若 #235 关闭而没有合并，需重新明确冻结依据。

[测试设计](TEST-DESIGN.md) · [当前报告](REPORT.md)
