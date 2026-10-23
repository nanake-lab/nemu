# P6-11 Codex 会话感知 — 报告

## 当前状态

**部分完成，PR #74 保留草稿。** 脚本、默认关闭的主进程接收端和本地验证已交付；面板设置接线与 P5-20 模拟器验收仍未完成，见 [BLOCKED.md](BLOCKED.md)。按当前 AGENTS.md，全部验收完成前不能转为可审查。等待用户处理范围请求及上游模拟器；不合并、不领取下一张卡。

工作区 `D:\Dev\nemu-wt\P6-11`，分支 `task/P6-11-codex-session`，从 fetch 后 `origin/main` 的 `0b5412e95caab447c70148c473e544dbab28c57c` 新建。最终验证的源码随后原样提交为 `f4cbb64db15dc1e71032d4b9fe26ef756d74f1eb`；最后一次 fetch 未发现 main 新提交。

未读取或修改用户真实 Codex 配置、隔离账号 CODEX_HOME、用户级或系统级设置。测试仅用 D 盘随机临时假配置与纯合成事件，没有启动 Codex、Electron、浏览器或其他用户应用，没有打开排练或正式存储。

## 交付物

- `tools/codex-hooks/`：显式目录安装、卸载、最小字段投影与发送、安装说明、三事件合成录制流。`pnpm codex-hooks:install` / `codex-hooks:uninstall` 必须同时给出 `--config-dir` 和 `--state-dir`，不根据环境寻找目标。
- `apps/desktop/src/codex-session/`：本机管道接收、严格字段校验、授权开关监听、每条消息重新检查授权、关闭与重启时在途事件拦截。向既有 AppBus 发布 `codex.session`，P6-02 负责后续观察转换。
- `apps/desktop/src/main/index.ts`：只追加导入和模块登记两行（AGENTS.md 第 2 节共享入口例外）。根清单只追加命令，锁文件仅登记新工作区包。
- [规格对照](../../docs/traceability/P6-11.md)、[设置接线范围请求](../../tasks/requests/P6-11-settings-integration.md)。

## 验收

最终本地验证于 2026-10-24 00:50:59–00:51:39（Australia/Perth）顺序执行。重负载窗口只交给一名 `routine_reporter`，按角色名调用；角色返回机械命令结果，主代理读取日志并作以下判断。

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `pnpm test:codex-hooks` | passed；1 文件、11 测试 | `evidence/hooks-passed-20261024-005059.log` |
| `pnpm lint` | passed；退出 0、无诊断 | `evidence/lint-passed-20261024-005059.log` |
| `pnpm typecheck` | passed；退出 0、无诊断 | `evidence/typecheck-passed-20261024-005059.log` |
| `pnpm test` | passed；46 文件、546 测试 | `evidence/test-passed-20261024-005059.log` |
| `pnpm build` | passed；退出 0；保留构建警告 | `evidence/build-passed-20261024-005059.log` |
| 重复安装 / 卸载，保留他人钩子、特殊 own key 与 config.toml | passed；临时假配置 | 专项测试 |
| 三事件最小字段；CLI 无目标拒绝；生成命令支持中文、空格、美元符号和引号 | passed；stdout / stderr 均空 | 专项测试 |
| 默认关闭、卸载后及在途连接 0 个事件；损坏钩子仍撤销授权 | passed；临时管道与服务 | 专项测试 |
| 未知事件、额外字段、无效 UTF-8、超长与多帧拒绝 | passed；即使 AppBus 关闭自身验证仍拒绝 | 专项测试 |
| 合成录制流 → 实际管道 → AppBus | passed；恰好一次 PermissionRequest | 专项测试；`fixtures/session-flow.jsonl` |
| 设置界面开启 / 关闭联动 | not_run / 未实现；目录范围待确认 | 设置范围请求 |
| P5-20 模拟器立即主动时机及全部门控 | not_run；上游模拟器不存在 | BLOCKED.md；不得以传输测试代替 |
| 本机 Electron / Playwright / 视觉 / GPU、真实 Codex 安装 | 未运行；本次禁止操作应用 / 真实配置 | 不作完成声明 |
| CI | 推送后以 PR 当前检查为准；Windows 对草稿跳过 | PR #74；不以本地测试冒充 CI |

此前失败保留：00:44 专项测试 2 失败 / 7 通过，原因是 Windows `--import` 收到普通盘符路径，已改为 `file:` URL；原始日志 `evidence/hooks-failed-20261024-004436.log`。00:48 lint 的两处 `preserve-caught-error` 失败已通过保留 error cause 修复，原始日志 `evidence/lint-failed-20261024-004847.log`。更早 00:42 的记录器把含冒号的命令名用作目录，重定向失败、未产生日志，测试是否启动与退出码无法确认；不计作产品测试失败或通过。

临时配置、缓存及原始执行日志在 `D:\Dev\cache\P6-11\` 的独立 run 目录。构建遵循现有项目命令生成工作区内的忽略产物。没有改测试断言、阈值、快照或黄金向量来消除失败。

## 规格对照

完整对照见 `docs/traceability/P6-11.md`。涉及 I02、I18、I26、I29：只发布观察；不持久化情绪或行为状态；不输出审批决定；测试仅用合成数据和临时目录。本卡不改 IDL、契约、黄金向量、主动边界、规格或差异登记。

## 新增依赖

没有新增第三方库或版本。新工作区包复用 `@nemu/contracts` 与已锁定的 `zod 4.6.7`（MIT）。脚本执行依赖现有 `tsx 4.21.0`（MIT）、Node 24，以及 Windows 自带 PowerShell；未安装额外运行时。安装钩子引用当前代码目录，移动目录需重新安装更新路径。

## 待用户确认

- [ ] **阻塞：** 批准设置接线的目录与接口范围，或明确调整卡范围；当前只有 CLI 开关与主进程服务。
- [ ] **阻塞：** P5-20 模拟器交付后补验一次立即主动时机和全部门控；当前不自行缩减验收。

## 后续建议（不在本任务范围内）

真实 Codex `/hooks` 信任由用户在检查脚本后操作。本卡没有验证真实账号、跨账号管道 ACL 或托管策略兼容性；接收端保持系统默认权限，未扩大管道可读写范围，没有修改用户或系统设置。
