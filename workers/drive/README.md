# drive-worker 离线评估接口

`nemu_drive.appraise` 供可信调用方在进程内调用。本批支持已接纳的文字（UserText）和语音（UserUtterance）观察。调用方传入经网关核实的 EventEnvelope、当前 ProposalBindingV2、AppraisalView 和实际加载的本地评估资源。源码包与 wheel 都使用从同一份 Echo IDL 生成的类型。这个接口目前由合成夹具调用。

- **路由与绑定。** 外层按事件种类及载荷的规范类型标签识别来源，核对事件 ID、日志位置和载荷引用的摘要一致性。文字和语音还须核对已接纳 z 和 c_core 的 CanonicalV1 摘要、历史前缀以及实际评估资源与绑定中的引用。事件来源认证、已接纳 z 的治理、所涉人识别均由可信调用方负责。
- **本地输入。** 回调只收到按 IDL 重建的 z、c_core 和按契约集合顺序排列的所涉人 ID。快状态坐标、m、θ、倾向强度、完整绑定、音频及原始载荷均不在回调接口内。调用者可提供 `asyncio` 事件循环时钟上的绝对截止时间。取消由调用方接收，超时和后端故障返回失败，一次请求只调用一次。
- **结果。** 回调针对 NatureVector 的四个契约字段分别交回一个数值或显式未知（`None`）。四维齐全且有效时返回带原绑定、生产者与版本的 NatureAppraisalRecord 提议；部分或全部未知时返回 IncompleteAppraisal，其中只保留已知的维度。缺字段、多字段、非法数值均返回 INVALID_OUTPUT。触摸、桌面和执行回执返回 UNSUPPORTED_SOURCE，回调调用次数为零。
- **提交。** 核心须把提议和其独立持有的当前绑定交给既有 validate_record。只有核心治理和外层事务才可写 H 或记录 A_DEGRADED。真实 m = 0、受约束解码、IPC 与端到端降级待后继联测。[核心记录检查](../../core/crates/core-runtime/src/binding.rs)

本包未附带默认评估器。测试中的 Replay 原样返回合成测值；DESIGN-APPRAISE-02 负责正式量尺，正式评分与三语质量验收保留在 P5-11。可在本目录运行 `uv run pytest`、`uv run ruff check .`、`uv run mypy --strict src tests` 和 `uv build`。[离线用例](tests/test_appraisal.py)
