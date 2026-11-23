# SYS17 契约前提

[verify_law_state.py](../../../core/contracts/verify_law_state.py) 已接入正常 `pnpm test:core-contracts`，通过证据见 [SYS21 完整日志](../SYS21/contracts-02.log)。

- 三个合成旧快照生成空槽新快照，x 与锚点不变；以实际编码摘要、长度建立前后引用及 unknown=true 的迁移记录。三语言产物一致。
- 每语言 36 项新旧双编码交叉解码拒绝、18 项原版本字节回读通过。
- 旧 θ 不补猜 P，不执行从 H 重算；新快照不冒充旧快照。

这不是存储迁移、故障原子性、备份恢复或真实重启体验验收；这些属于 P5-75。本卡没有读取用户谱系。
