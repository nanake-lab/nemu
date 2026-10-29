# P5-03 / P5-22 合并后复验

## 结果

B2 解除。#57 合并提交 b6eb8721b420b4aa168e0c27789f63307959f84c；
首次 fetch/merge main=541be9e，本分支合并提交 952f0ef。
结束前又合并 main=0b5412e（P2-05），合并提交 d934850。
两版 main 的 core/、packages/contracts/、pnpm-lock.yaml 相同，完整检查受测输入不变。
B1 仍按用户证据通过；B3 等新契约卡，完整服务/SYS01/真实断电仍 not_run。保持草稿，报告后停下。

## 本轮实现与约束

- 生成类型走原有 u32 LE + protobuf 帧；不新建类型标签、消息外壳或 RPC 分派。
- 六种旧类型仍由 core-types 解码历史字节，live Framed 收发均拒绝。
- 八种新消息共 31 个合法黄金变体经帧往返，逐字节匹配冻结 protobuf 和规范化 expected.value。
  ReadoutV2 保留获批 v1/v2 oneof 迁移；RunStateV2 拒绝 EXPERIMENT；notice 无回应/投票；growth.control 只有 pause/resume。
- QueryV2/QueryResultV2 核对 query_id；growth/items 互斥、完整保留 L 和所有状态，不变成记忆投影。
- 审批核对原样实际载荷/SHA256/approval ID/action ID/有效期，拒绝 DENY 和 T3 信任。
  只返回不可 Clone/Debug/Serialize 的一次性本地快照，消费时重查时间并只借出同一不可变载荷；失败亦消耗。
  这不是身份或执行凭证。P5-05 和未来可信分派层仍负责认证、权限、关联来源、持久审计与执行。
- 九类 EventEnvelope 载荷校验继续保留；控制消息没有被塞进 observation 或用于成长。
- sha2 0.11.1 由既有锁定依赖转为 gateway 直接引用（MIT OR Apache-2.0）；无新版本或传递依赖。
- 本轮未改 IDL、黄金、生成器、core-store 或原接口请求；只有合并 main 带入它们的既有修订。

## 验证与证据

| 检查 | 结果 | 原始证据 |
| --- | --- | --- |
| ../check.ps1：fmt、clippy workspace all-targets -D warnings、test workspace 串行 | 106 passed、0 failed；含两项 compile-fail；gateway 21、core-store 31（1008.67 秒） | workspace-checks.log |
| check-contracts.ps1：冻结安装、uv locked、compat --ref main、终端契约、三语言 | 整组退出 0；350 契约测试；三语言各 10,766；官方 protoc 445；8 schema；0 stale；基线/current/main 0 违例 | contract-checks-final.log |
| 旧消息/新消息、查询与审批正反例、ACL/DPAPI、SYS01 结构子集 | 全部随网关 21 项通过 | workspace-checks.log |

完整矩阵使用 D 盘 TemporaryRoot/tempfile 合成库，未打开用户库；不涉及真实成长或 GPU/性能/外观验收。
Rust/Cargo 1.99.0，Node 24.21.0，pnpm 12.11.2。构建、Corepack/Node/pnpm/UV 缓存和合成临时目录均在 D 盘。
core/target 为指向 D:/Dev/cache/cargo-target/P5-03 的本地 junction，满足既有 verify.py 的固定 probe 路径；
没有为了本卡改契约工具。覆盖率输出复制到本目录，测试生成的原 P0-02 汇总已恢复，不混入卡外改动。

source-manifest.json 为受测代码、IDL 和黄金的 LF 归一 SHA256；successor-vector-counts.json 为黄金覆盖计数。
首次完整测试针对集合原始顺序的预期不正确：gateway-initial.log 保留 20 passed/1 failed；修正为冻结
expected.value，并同时核对冻结 protobuf，没有改黄金、规则或放宽断言。
首次契约脚本因不支持 --cache-dir 参数退出，保留 contract-checks.log；改为环境配置后整组复跑成功。
既有失败原始证据均保留。

末次仅合并不涉及 Rust/契约的 P2-05 后，静态检查、gateway 与 origin/main 兼容再次核验；
最后结果与输入一致性见 sync-checks.json。没有重跑输入相同的约 17 分钟存储矩阵。
