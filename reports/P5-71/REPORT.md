验证档位：重档。

# P5-71 新规律机制实验 — 报告

## 结果

部分完成：完整矩阵适配器与输入已物化，工具检查通过；正式矩阵尚未启动，机制结论仍为 not_run。用户已解除原四小时停止门，批准按冻结 v2 全量执行一次、上限 16 小时且不做工具优化。正式运行前绑定见 RUN-MANIFEST；最终结果与证据待本次运行补入。

## 实现与验证

[逐项实现对照](IMPLEMENTATION.md)。仅修改 core-kernel/tests 与 reports/P5-71；正常测试构建包含全部适配器，无新增依赖、生产接口或默认参数。

| 检查 | 结果 | 日志（待私有归档） |
| --- | --- | --- |
| cargo test -p core-kernel | 退出 0；单元 273 passed / 1 既有 ignored，集成 4 passed，doc-tests 0 | full-kernel-test-01.log |
| cargo clippy -p core-kernel --all-targets -- -D warnings | 退出 0 | full-clippy-01.log |
| cargo fmt --all -- --check | 退出 0 | full-fmt-check-01.log |
| ruff / mypy --strict | 通过，23 个 Python 文件 | full-ruff-04.log / full-mypy-07.log |
| 完整输入物化 | 完成，未执行规律；旧版准备文件保留 | full-materialize-01/02.log |

本轮工具验证的失败均保留：TextIOWrapper 被深拷贝、G 资源封装字段混入严格参数消息、G 枚举字符串与 IDL 数字不一致。分别修复测试工具后重跑相应夹具；最终全包命令成功。此前 probe-01 引用绑定失败、Python 类型检查/工具构建失败仍在原证据中，不改写为成功。

## 结论边界

冻结预注册 e5854262 未修改。固定 ρ 与内部候选边界 DUT 仍 not_run；真实时间、消费、治理、恢复、闭环和真实体验仍按 v2 交 P5-72/73/74/75/76/77。工具测试通过不算正式矩阵 passed，也不代表整卡完成。

此前六条有效探针 27.5584 秒、已估部分约 10.0 小时，仅作成本依据；原始证据提交 707bee9dfc656626b617f1df6822b4cf20712c49，见 [证据索引](EVIDENCE-INDEX.md)。本轮原始证据暂存 D:/Dev/cache/P5-71，执行后归档至私有证据库，公开仓库只保留报告、汇总及索引。

## 下一步

登记本次实际代码、输入/资源/原语/二进制哈希及分层计算量后串行执行一次。到 16 小时、工具故障或中断即保留前缀，其余 not_run；不自动续跑或改规模。完成后推送报告并在 PR 写“请审查”，保持 Draft / needs-user，不合并、不取下一卡，独立交叉审查由用户另行安排。
