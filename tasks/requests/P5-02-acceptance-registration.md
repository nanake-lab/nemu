# P5-02：中央 W16 验收入口登记

- 原因：P5-02 目录授权仅 `core/crates/core-store/`；Echo Core 技能的统一验收目录
  `core/tests/acceptance/` 由后续 P5-18 / W16 汇总，不在本卡代码范围。
- 本卡已在自身 crate 内提供存储正例、拒绝用例、真实进程崩溃恢复、三谱系隔离和
  磁盘/容量故障测试。完整命令：`cargo test -p core-store --locked -- --test-threads=1 --nocapture`。
- 期望接口：W16 按 `docs/traceability/P5-02.md` 的映射注册 16.1、16.13、D10.1、
  D10.2、X03、AT01–AT04、SYS22、I29、AC2-22 的存储子集，读取每个编号的 result.json。
- 本请求不要求改变阈值或跳过本卡测试，也不宣称 W16、成长有效性或系统级验收完成。
