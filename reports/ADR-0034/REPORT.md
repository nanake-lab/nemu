轻档：本次只修改 ADR、任务卡与报告，不执行下载或测量。

# ADR 0034 — 交付报告

## 当前状态

文档与卡已起草，PR 标记 `needs-user`，等待审查及用户合并。P0-12 尚未执行，模型切换尚未生效；约 9 GB 官方权重下载与 GPU 安静窗口须在实测前由用户批准/安排。本次没有运行失败或实施阻塞。

## 改动

- [ADR 0034](../../docs/adr/0034-qwen35-4b-conditional-carrier.md)：记录 2026-11-01 决定、条件生效与两级回退。
- [ADR 0031](../../docs/adr/0031-carrier-engine-and-injection.md)：修订条件载体与清除 KV/循环状态要求。
- [P0-12](../../tasks/P0-foundation.md#p0-12-视觉载体条件切换实测)、[cards.json](../../tasks/cards.json)：新增 E2 实测卡，依赖 P0-10；P5-13 增加 P0-12 依赖。
- [P5-13 卡头](../../tasks/P5-echo-core.md#p5-13-模型载体适配器w12)：同步依赖，并补齐 cards.json 已有的 P0-10。

位置 / 依据 / 新读法：ADR 0031 §1 / 用户 2026-11-01 决定、ADR 0034 §1–2 / Qwen3.5-4B 实测通过才切换，失败依次复测 Qwen3-VL-4B-Instruct、保留旧载体。

位置 / 依据 / 新读法：ADR 0031 §3 / 同日决定、ADR 0034 §3 / 清 KV 扩展为清 KV 与循环状态，重新 prefill，结果须由 P0-12 确认。

## 检查

- `git diff --check origin/main`：通过。
- Python `json.loads` 完整解析 `tasks/cards.json`：通过；新增依赖与 E2 引用存在，任务依赖无环。
- Python 检查本次新增/修改 Markdown 相对链接：目标文件均存在。
- 编号检查：最新 origin/main 与全部开放 PR 未占用 ADR 0034 / P0-12，无顺延。
- 按用户要求不等待或报告 CI，不运行全工作区测试，不做证据归档或哈希核对。

新模型所有实测均为 `not_run`，不以本次文档检查签发模型通过。下一步为审查；本会话不执行 P0-12。
