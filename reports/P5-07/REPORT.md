# P5-07 IRL worker 与 codex-bridge 基础层 — 报告

## 当前状态
部分完成（见 [BLOCKED.md](BLOCKED.md)）。PR #96 保持 **draft + needs-user**，未达到转为可审查或合并的条件。交付不重叠的 bridge 基础库；worker、实际 IPC 与正式模型验收未完成。源代码核验提交为 `6e8fb8c5547560673d1dc508b724b7f7be50f8e4`，已经 merge 主线 `2bc04bad4902657eef49d3225cafa7945e7cfae6` 并重跑检查。当前 head 的 CI 以 PR 实时检查为准，不拿旧提交或本地通过替代。接下来按用户要求处理 P6-17 的目录阻塞，不另取其他卡。

P5-34 的开放 PR #95 占用 `workers/*/generated/`，与本卡 `workers/irl/` 父目录重叠；N0 把 IRL 部署参数留待 P0-10 后以 ADR 决定；端点/请求合同与冻结的三语人工金标准也仍缺失。

## 交付物
- `executors/codex-bridge/src/base/`：独立 home 独占锁、CLI 版本校验、P0-08 隔离配置、有限 JSON-RPC 帧与截止时间、线程/轮次/流式/steer/interrupt、逐轮零工具/零 MCP 守卫、按档位记录用量、断线不盲目重发。它是库，没有注册为生产服务。
- `base/generated/`：CLI **0.162.1** 的 `generate-ts --experimental` 原样类型的传递闭包，45 文件，逐文件 SHA256 在 `MANIFEST.json` 与核验记录中。
- `package.json`、`pnpm-lock.yaml`：依 AGENTS.md §2 共享清单例外，只增加仓库已使用版本的 Zod 根直接依赖；未升级既有依赖。
- `reports/P5-07/`：报告、阻塞说明、原始检查与指标状态。规格对照放在本报告，遵守用户的目录限制。

**worker 未交付。** 首次目录核对漏看 P5-34 生成目标，曾本地做过未提交候选及 23 项 Python 测试。发现父子目录重叠后，逐文件校验 SHA256，封存到 `D:/Dev/cache/P5-07/deferred-worker/`，恢复分支中的 worker 文件；候选未提交或推送。早期 Python 证据不计入本 PR 验收。

## 验收
| 命令或检查（含 Echo Core 验收编号） | 结果 | 证据 |
| --- | --- | --- |
| `pnpm lint` | passed | `verification/final-bytes/lint.log` 与同名 JSON |
| `pnpm typecheck` | passed | `verification/final-bytes/typecheck.log` 与同名 JSON |
| `pnpm test` | passed：54 文件、598 项 | `verification/final-bytes/test.log` 与同名 JSON |
| bridge 编译到 D 盘缓存 | passed | `verification/final-bytes/bridge-build.log`；JSON 中含完整命令 |
| bridge 控制/隔离守卫 | 局部 passed：18 项，合成协议帧与临时 home 锁 | `home.test.ts`、`replay.test.ts`；包含于上述 598 项 |
| 同版生成类型哈希 | passed：45/45，0 不一致 | `verification/generated-hashes.json` |
| 修改目录检查 | passed：0 越界；worker 最终 0 修改 | `verification/scope.json`；根清单为共享例外 |
| 真实 provider 录制、真实服务启动/重连 | not_run；合成回放不能替代 | `BLOCKED.md`；未使用认证或消费推理额度 |
| 命名管道服务、实际双后端客户端 | not_run；没有私有线协议或未认证 listener | `BLOCKED.md` 第 2 项 |
| 三语各 100 条：schema ≥95%、人工语义准确率 | not_run | `metrics.json` |
| 新后端 P50/P90 与 P0-08 比较 | not_run，均为 null | `metrics.json`；历史隔离 low 为 6524.99/9269.47 ms，非部署值 |
| ADR 0014 §5：解释/认识论混淆矩阵、语言差 | not_run，缺人工金标准 | `metrics.json` |
| SYS02 联测 | not_run；仅 bridge 局部工具/权限拒绝测试 | `SYS02/result.json` |

失败原始记录保留：`attempt-01/pytest.log` 是误从仓库根运行 pytest，收集到其他项目后有 6 项收集错误；更正 cwd 后缓存候选的 23 项通过。`attempt-01/mypy.log` 的 8 项为候选测试注解，修正后通过（`attempt-02/`）。初次离线安装缺已有 koffi tarball，诊断后按同一锁版本在线安装成功。`final/bridge-build-post-main.log` 是 PowerShell 数组未正确 splat 的调用失败，正确命令后通过。最终有效结果只取 `final-bytes/`，旧结果不覆盖。

## 规格对照（Echo Core 相关的卡必填）
| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| ADR 0004 §1；P0-08 复审移交：独立 home、固定版本、隔离能力面 | `home.ts`、`process.ts`、`isolation.ts` | 版本不符、重复认领、继承环境、未受限 catalog 均拒绝 |
| ADR 0004 §1：同版类型、线程/轮次/流式控制 | `generated/`、`protocol.ts`、`rpc.ts`、`bridge.ts` | 早到 completion、delta、steer 的 expectedTurnId、interrupt、身份错配、有限帧 |
| P0-08 REVIEW-R1-REPORT：逐轮零工具/零 MCP | `Bridge.run`、`checkInventory` | raw/typed 工具、审批、模型替换、非空/分页/未知库存均失败；completion 后迟到工具也失败 |
| ADR 0004 §3、CONTRACTS §6：不自动同意请求 | `RpcClient.accept` | server request 返回错误并使整轮失败；助手审批执行留给 P6-06 |
| ADR 0004 §4–5：配置档位、用量 | `base/index.ts`、`Bridge.usage` | 显式配置；未知 token 为 null，不重复加总子集；正式 helpers 仍缺 |
| S §21、O §16.3、I16：失败有界，不盲目重发 | `rpc.ts`、`bridge.ts`、`process.ts` | 截止时间、断线、额度耗尽；旧请求只发送一次，新客户端才恢复 |
| F §3.3/3.5；D 第一篇 §4.2/5.3；v2.0 §2.4、EV6 | worker 未交付 | 目录阻塞；缓存候选不签发 IRL/m=0 后端结论 |
| I02、I18、I29 | 无权威存储 API；测试仅合成输入和临时目录；用量缓存显式限容、不含输出文本 | 文件锁/回放测试；未运行 Low IL 或正式存储测试，不宣称 OS 层隔离已验收 |

无 Echo Core IDL、黄金向量、ADR 或差异登记变更。未把文字输出称为严格载体调制。协议参考：[OpenAI App Server 官方文档](https://learn.chatgpt.com/docs/app-server)，参数以本机锁定版本生成文件为准。Context7 当时额度不可用，改查官方文档与既有 P0-08 源码。

## 新增依赖
| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| zod（新增根直接依赖，仓库已有同版） | 4.6.7 | MIT | JSON-RPC/配置校验 |

Codex CLI 0.162.1 为既有外部程序，未安装或升级。实际 Node 24.21.0、pnpm 12.11.2、uv 0.12.15 见 `verification/tool-versions.json`。手写文件均小于 400 行。

## 待用户确认
- [ ] （阻塞）#95 结束或用户明确批准目录互斥例外后恢复 worker。
- [ ] （阻塞）按 N0 决定，P0-10 后确定 IRL 架构组合、默认后端、截止时间与 helpers 配置。
- [ ] （阻塞）所属契约/端点卡补齐 IPC、认证、对象及本地端点合同，完成真实连接与 SYS02。
- [ ] （阻塞）冻结三语人工金标准，完成正式 schema、准确率、偏差与延迟验收。
- [ ] （阻塞）执行器范围需用户安排交叉审查；请审查。其余验收尚未完成，当前不能转为可审查或合并。

## 后续建议（不在本任务范围内）
- P6-04/P6-06 复用基础库时，为各自权限域另行接入；P6-06 的授权工具线程不能用 talk-only 策略代替行动闸门。
- 端点契约须给出实际 c_core/注册表对象的有界解析和绑定，不仅传不可解析引用，由所属卡完成。
