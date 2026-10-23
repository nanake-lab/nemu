# P5-03 入口与网关（W18）—报告

## 当前状态

**部分完成，保持 draft / needs-user。** 已 fetch 并合并 origin/main
`0b5412e95caab447c70148c473e544dbab28c57c`（最终合并提交 `d934850`）。
P5-22 / #57 已合并，后继消息校验及本轮本卡全部检查通过，B2 已解除，结果见下表。
B1 第二账户检查已通过；B3 的公开接口由新的契约卡处理，本卡不扩大范围。
当前交付是边界组件，完整认证网关、完整 SYS01 和真实断电验收尚未完成。
更新报告后按用户要求停下，不转为可审查、不请求最终审查、不合并。

## 交付物

- `core/crates/gateway/`：限长/限时帧、CONTRACTS 4.2 九类事件验证、DPAPI 安装令牌、当前用户管道、core-store 接收适配。
- `terminal.rs`：P5-22 生成类型的查询关联与成长内容隔离、审批原样载荷/SHA256/ID/有效期/T3 约束；没有新线协议。
- `tests/terminal.rs`：八种新消息的全部合法黄金变体及冻结 protobuf 字节、旧六类型的历史解码/live 拒绝、非法消息与上下文正反例。
- 实际 Windows 管道到 core-store 的 SYS01 结构集成：重投/冲突不改变快照、锚点和账本；跨重连 e2/e1 逆序延迟回执；未提交关闭重开。
- `examples/acl_probe.rs`、`second-account/`：用户的实际跨账户结果原文、来源边界及手工合成探针。
- `check.ps1`、`post-P5-22/check-contracts.ps1`：本卡完整 Rust 与契约复验；缓存、临时目录、venv 均指向 D 盘。
- Windows CI 仅追加 W18 检查和证据上传；草稿时自动跳过。接口请求 `tasks/requests/P5-03-gateway-interfaces.md` 本轮未修改。

## 认证、ACL 与权限（必审）

- DPAPI 当前用户范围、Windows CSPRNG 32 字节令牌、只保存密文；CREATE_NEW 不覆盖既有或损坏文件；令牌无 Debug/Serialize，零化缓冲、常量时间比较，日志不含凭据。
- 管道和令牌文件 DACL 只含当前 SID、保护继承；管道拒绝远程、初次 first-instance。断开时先建相同 ACL 的新句柄再关旧句柄，保持名称连续占用；最多两个内核实例轮换、逐会话处理。20 次双向重连逐次检查实际 DACL 和名称抢占拒绝。
- 第二账户：用户在 Windows 10 用 nanake 服务端/nemu 客户端，收到 ERROR_ACCESS_DENIED (5)，exit 0；原文见 second-account/user-message.txt。未附完整窗口逐行输出，不补造。结果来自轮换修复前；本轮未代跑跨账户测试，ACL 构造不变。
- T1 标题/Codex 会话权限逐事件传入，不在会话中缓存；撤权反例持续保留。
- `CheckedApproval` 是私有持有原样载荷的一次性本地校验快照；无 Clone/Debug/Serialize。核对实际待执行载荷、SHA256、approval/action ID、有效期，拒绝 DENY/T3 信任；消费时重查有效期与时钟回退，回调失败也消耗快照。
  **这不是身份认证、权限授予或执行凭证。** P5-05 的真实行动权威/一次性凭证/审计仍需接入。未来可信分派层必须调用查询和审批上下文校验；单独 Framed 解码不提供关联认证。
- 后继查询保留完整 L 与所有状态，growth/items 互斥；仅用于 user_view，不能送进 c_core/c_task。Notice 无投票字段，GrowthControl 仅 pause/resume；都不执行任何管理命令。

## 验收（本轮）

| 命令 / 检查 | 当前结果 | 证据 |
| --- | --- | --- |
| cargo fmt、workspace all-targets clippy -D warnings、完整 workspace test（串行） | 通过；106 passed、0 failed（含两项 compile-fail）；core-store 31 项用时 1008.67 秒 | post-P5-22/workspace-checks.log |
| pnpm test:contracts | 350 passed，0 failed | post-P5-22/contract-checks-final.log |
| pnpm test:core-contracts、compat --ref origin/main | Rust/Python/TypeScript 各 10,766 例通过；protoc 445；8 schema 回归；0 stale；全部基线/current/main 0 违例 | post-P5-22/contract-checks-final.log |
| 八种新消息全部黄金变体、旧类型 live 拒绝、查询/审批上下文 | 31 个合法黄金变体全部通过；新增五项回归随 gateway 21 项全部通过 | post-P5-22/workspace-checks.log、successor-vector-counts.json |
| 第二 Windows 账户拒绝连接 | passed（用户提供的实际结果） | second-account/ |
| SYS01 真实管道/存储结构子集 | 本轮复验通过；不代替完整服务/真实断电 | post-P5-22/workspace-checks.log |
| 完整 SYS01、真实断电、完整认证/上传/持久非法审计 | not_run，B3 阻塞 | BLOCKED.md |

首轮新黄金回读测试失败：把集合输入原顺序当成回读预期。已使用冻结黄金的规范化 expected.value，
并核对 expected.protobuf 字节；未改黄金、编码或集合规则，原始失败保留 post-P5-22/gateway-initial.log。
此前的重连 Win32 233、夹具和其他失败仍原样保留在 continuation-1/ 与原日志，不用成功覆盖失败。
契约复验首次停在 pnpm 不支持 `--cache-dir` 命令行参数（contract-checks.log）；改为环境配置并保留
相同版本/冻结锁文件后，整组脚本重跑通过（contract-checks-final.log）。工具、venv 和缓存全部在 D 盘。
完整矩阵针对合并 541be9e 后的源文件；末次同步又合并 P2-05。两版主线间 core/、packages/contracts/
和 pnpm-lock.yaml 无变化，受测输入相同；合并后再核对静态检查、gateway 与 main 兼容，见 post-P5-22/REPORT.md。
本轮不跑 GPU、外观或性能测量，不声称安静测量；所有测试仅用 D 盘 TemporaryRoot/tempfile 合成库。

## 规格对照与依赖

规格映射见 docs/traceability/P5-03.md。I01/I04 由 core-store 去重与断言验证；I02/I03/I18/I21
保持唯一存储和写能力；I26 由生成类型与审批快照边界验证（完整行动闸门未完成）；I29 由谱系类型、
实际实验 bind/ingress 拒绝和 RunStateV2 的 TerminalLineage 检查。

P5-22 合并带入既有契约及生成产物，本卡没有自行修改 IDL、编解码、黄金向量、core-store、规律或阈值；
无新差异登记。ReadoutV2 的 v1/v2 oneof 都按获批迁移契约解码；旧独立 Readout 等仍可由 core-types
解码历史数据，但不能经 live Framed 收发。没有自造上传/认证 envelope 或第二套 D 计数/去重。

现有依赖：tokio 1.54.1（MIT）、zeroize 1.8.2（Apache-2.0 OR MIT）、subtle 2.6.1（BSD-3-Clause）、
windows-sys 0.61.2（MIT OR Apache-2.0）、serde_json 1.0.154（MIT OR Apache-2.0）、工作区 core-types/core-store。
本轮直接引用已锁定的 sha2 0.11.1（MIT OR Apache-2.0），用于 CONTRACTS §6 的原样载荷 SHA256；
没有新增版本或传递依赖。测试使用已锁定 tempfile 3.27.0。完整原始依赖/许可证登记可从本报告历史版本追溯。
初始新传递依赖为 bytes 1.12.1、mio 1.2.4、tokio-macros 2.7.2（MIT）、socket2 0.6.5
（MIT OR Apache-2.0）与 wasi 0.11.1+wasi-snapshot-preview1（Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT）。

## 待用户安排与下一步

- [x] B1：实际第二账户拒绝连接通过，按用户提供证据登记。
- [x] B2：#57 已合并，后继校验及本轮本卡全部检查通过。
- [ ] B3：新的契约卡处理认证/上传分派、D 尾位置、持久拒绝审计、有类型背压接口。

B3 接口可用前保持草稿；本轮报告后停下，等待用户安排，不自行跨卡补协议或权威存储。
