# P5-22 终端契约变更 — 报告

## 当前状态

**完成。** [PR #57](https://github.com/nanake-lab/nemu/pull/57) 已由用户于 2026-10-23 合并，合并提交 `b6eb8721b420b4aa168e0c27789f63307959f84c`。A1 修复经用户另开的 Codex 新会话[复审通过](https://github.com/nanake-lab/nemu/pull/57#issuecomment-6360980179)，按 2026-10-23 规则代替原 Claude 审查要求；B1、B2、D1 已在前轮关闭。Ubuntu 与 Windows CI 均通过，原 runner 离线阻塞已解除。用户合并即批准本次契约、主动边界候选及 V2-51–54，无待确认的合并阻塞。

本次仅在 D:\Dev\nemu-wt\P5-22 的新分支 `task/P5-22-report-closeout` 更新报告，未改代码、契约、配置或原始证据；按用户要求开 PR 后停下，不认领下一张卡。

## 交付物

- `docs/CONTRACTS.md` v2.2：身体读出、主动边界、成长通知/查询/控制、实际行动审批和社交权限。
- `core/contracts/terminal.proto` 及生成产物：保留旧消息/字节，以后继消息实现 v2；权威主动边界资源及其 JSON 投影由 IDL 生成。
- `packages/contracts/`：身体 v2、通知/控制、一次性审批快照、完整 L 查询返回、严格资源字典校验。合法特殊自有键无损保留，非法值明确拒绝。
- `config/opportunities.json`：仅含种类、冷却、门控与预算，无触发脚本；候选随用户合并获批。
- 黄金向量目前为 445 合法、457 非法、18 Unicode；ReadoutV2 的 oneof 按 ADR 0007 编码，旧向量保留。
- [REVISION-1.md](REVISION-1.md)、[REVISION-2.md](REVISION-2.md)：修订实现、配套工具修改、逐项回归及字节哈希；[BLOCKED.md](BLOCKED.md)：原阻塞解除说明。

## 验收

下表引用原实现提交的验收；本次仅报告收尾，不重跑代码测试。原始日志、失败记录和 JSON 证据全部保留。

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| 本机完整检查 | 26 项均有成功结果；迁移后 Python 启动器失效已重建环境并复跑整组 14 项通过 | revision-2-final-checks.json；revision-2-checks/；revision-2-python-checks/ |
| pnpm lint/typecheck/test/build | 通过；33 文件、481 测试 | revision-2-checks/{lint,typecheck,unit,build}.log |
| pnpm test:contracts | 350 测试通过 | revision-2-checks/contracts.log |
| D10.8 / SYS21 / 三语言契约 | Rust/Python/TypeScript 各 10,766 例通过；protoc 445 向量通过 | revision-2-checks/core-contracts.log |
| 生成与兼容门禁 | 0 stale；p5-01/p5-21/p5-32/current/origin/main 0 违例；8 个 schema 回归通过 | revision-2-checks/{generate,compat,parser}.log |
| Rust fmt/clippy/workspace；契约工具和三个 worker 静态检查/测试 | 全部通过，包括 core-store 崩溃恢复矩阵 | revision-2-checks/；revision-2-python-checks/ |
| 冻结字节与摘要 | 原 443/457/18 前缀不变；每语言 1,816 查询、469 拒绝、0 差异；两个 A1 向量另行核对一致 | revision-2-frozen-byte-comparison.json；revision-2-resource-vectors.json |
| 新 Codex 会话 A1 复审 | 通过，A1 关闭；独立扩展合法/拒绝矩阵、三语言及旧向量核对通过 | [复审结论](https://github.com/nanake-lab/nemu/pull/57#issuecomment-6360980179) |
| Ubuntu CI | success，受测 head a57c8b1 | [run 39910674806](https://github.com/nanake-lab/nemu/actions/runs/39910674806) |
| Windows CI | success，受测 head a57c8b1；原离线阻塞解除 | [run 39928691322](https://github.com/nanake-lab/nemu/actions/runs/39928691322) |

首轮施工中的 TypeScript、Rust 注册表失败，以及修订二环境迁移造成的 Python 启动器失败，均保留在原日志中；后续成功结果未覆盖它们。具体修复及复跑记录见两份修订报告。旧 main=91b31ed 的 396/409/18 核对、修订一的 433/448/18 核对及 oneof 独立 oracle 证据也保留。

## 规格对照与配套修改

逐条对应见 `docs/traceability/P5-22.md`，包括 ADR 0007、I18、I26、I29、AC2-20 的接口部分。V2-51–54 及其补充随用户合并获批。

oneof 只实现 ADR 0007 已规定的 u32 选中号后接值；缺失、多选、未知号和尾随字节拒绝。生成器、三语言编解码与校验器、浏览器资源投影、兼容门禁和夹具等配套修改已与契约一起经过审查，文件与必要性详见 REVISION-1 / REVISION-2 的「配套修改」。TypeScript runtime 原有 >400 行及 generate.py 415 行的组织理由保留在 REVISION-1；生成产物大小由 IDL 决定。

## 新增依赖

无新增运行时或开发依赖。

## 待用户确认

- [x] 交叉审查通过：原 B1/B2/D1 已关闭；A1 经新 Codex 会话复审通过，按 2026-10-23 规则代替 Claude 审查。
- [x] 用户批准契约、opportunities 主动边界候选和 V2-51–54：用户于 2026-10-23 合并 #57，合并即批准。

BodyReadout v1 兼容保留和生产停用安排属于获批的 V2-51；P5-19 只发 v2、P3-06 迁移舞台仍由对应任务完成。

## 迁移、限制与后续建议

- 未打开用户正式/排练存储或运行迁移；旧字节保留，新消息使用新标签。部署时仍须依 ADR 0007 备份并升级协议双方，回退保留新对象原标签；新审批不能降级到旧无载荷入口。
- 此卡完成契约和接口验收，不代签后续成长、社交、生产读出或 GPU/外观验收。运行时分别由 P5-05、P5-12、P5-19、P5-29、P3-06、P6-14/P6-15 等卡承担。
- 继承的非阻塞 C1(a) 单向 supersession 门禁缺口仍是后续事项，不因本卡完成而视为修复。
