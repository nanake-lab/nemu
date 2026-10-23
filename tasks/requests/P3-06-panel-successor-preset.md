# P3-06：补登记舞台后继命令样例

首次 `pnpm typecheck` 与 `pnpm test` 发现 `apps/panel/src/debug/schema-validator.ts` 的 `STAGE_COMMAND_PRESETS` 使用 `Record<StageCommandType, ...>` 穷举契约消息。新增 `stage.readout.v2` 后缺少该项，使类型检查和既有穷举测试失败。原始日志在 `reports/P3-06/checks/typecheck-first.log`、`test-first.log`。

请求仅在该表追加以下样例，保留全部旧项，不重构面板，不修改测试：

```ts
'stage.readout.v2': { readout: { version: 'readout-v2-preview', facing: 0.5, tension: 0, energy: 0.5 } },
```

2026-10-24 用户批准：只追加这一条中性样例，旧样例不动；须通过后继 schema 校验，并在报告列明卡外改动。
