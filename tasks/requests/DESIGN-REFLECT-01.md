# DESIGN-REFLECT-01：反思触发的重新评价

- 来源：用户 2026-11-12（Australia/Perth）在本任务中的直接指派。
- 状态：设计草案已交 [PR #169](https://github.com/nanake-lab/nemu/pull/169)；只交 ADR 提议，需用户决定与独立审查。
- 分支：`task/DESIGN-REFLECT-01-reappraisal`。
- 目录：新增本请求、`docs/adr/0037-reflection-triggered-reappraisal.md`、`reports/DESIGN-REFLECT-01/`；仅追加 `docs/traceability/diff-register.md` 的候选差异。
- 编号预占：ADR 0037；V2-152 至 V2-155。开工已核对 origin/main `02372ff6` 与开放 PR #45、#111、#116、#144、#154、#160、#164；#164 使用 ADR 0036 / V2-151，其余无更高占用。

## 交付与边界

1. 至少三种可行机制，逐项说明触发、记录、G/C 路径、防循环、EV1–EV10、成本与虚构过去风险。
2. 推荐一种，解释其余方案不采用的原因；区分既定约束、代理建议与用户待决定项。
3. ADR 状态为提议；不改规格、契约、代码或既有任务卡；差异只作候选。
4. 设计合成世界中的「需要回想才明白」、重复反思与反证试验；机制、判断质量、实际选择分层验证，不运行实验。
5. 文档格式与相对链接检查；PR 标 `needs-user`，写「请审查」后停止，不合并、不领取下一卡。

## 依据

[v2.0](../../docs/echo-core/echo-core-v2.0-first-principles.md) 全文，重点 §§2、4–8、14、17；[ADR 0026](../../docs/adr/0026-continuous-cognition.md) §§5–6；[ADR 0032](../../docs/adr/0032-presets-as-starting-points.md) §§1、3、5–6；[ADR 0035](../../docs/adr/0035-law-definitions-d07-d09-d11.md) §§3–5、8–9；P5-70 的 F 替换边界。
