重档，含严格调制、正式模型与 GPU 验收。

# P5-13 模型载体适配器

部分完成。用户更正了派卡材料第 3 条，采用已生效的 ADR 0034：Qwen3.5-4B，Q4_K_M、不思考模式，mmproj F16 和资产哈希以 [P0-12](../P0-12/REPORT.md) 为准。

- 已用 P5-42 的生成类型和 CanonicalV1 校验器消费信封与资源。工作包 pytest 12 项、Ruff 和 strict mypy 通过。结构夹具覆盖资源错配、预算、方向增减及拒绝路径。
- 对模型 14 个文件、GGUF 3 个产物和引擎 55 个文件复核 SHA-256，72/72 相符。[私有证据](https://github.com/nanake-lab/nemu-evidence/blob/e9e72e0860fa6040f6e3b896fb4a07857076382f/P5-13/asset-check.json)，SHA-256 为 `69a698e7e500d63d94f2f5e7737fafc27a6fc34254331b11a0bfa3dccb054f6b`。
- [IRL 冻结输入](../../workers/model-adapter/tests/data/irl-inputs-v1.jsonl) SHA-256 为 `da05c8d65dffc593d22dc4b5560f2601a3790ae736b99b86789c7974a8a7a3bf`。中、英、日各 100 条。每种语言同义改写 12，否定、条件、愿望、引语、用户自报、纠正、歧义指代、新造词各 11。每行有稳定 ID，annotation 留空。输入由 P5-13 生成，未经人工标注，只用于 schema 合格率和延迟。
- 原生库已编译。J01、J05 真实缓存对照、AT13、显存、延迟、300 条 IRL、三语身份、生产回执待验证。
