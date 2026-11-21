重档，含严格调制、正式模型与 GPU 验收。

# P5-13 模型载体适配器

部分完成。载体能接收核心发布的信封，再为本地说话和结构化判断建立独立请求。用户更正了派卡材料第 3 条，采用已生效的 ADR 0034，使用 Qwen3.5-4B、Q4_K_M、不思考模式和 F16 mmproj，资产哈希以 [P0-12](../P0-12/REPORT.md) 为准。RTX 3080 的正式结论待实测。

- 已用 P5-42 的生成类型和 CanonicalV1 校验器消费信封与资源。Windows 本机 `uv run --locked pytest -q` 通过 16 项，`ruff check .`、Windows/Linux 两种目标平台的 strict mypy 和 `uv build` 均通过。结构夹具覆盖资源错配、方向增减、请求取消与回执绑定。[CPU 核验日志](https://github.com/nanake-lab/nemu-evidence/blob/c15db63695116f210a3be62bd5e54143b5875a84/P5-13/cpu-preflight.log)，SHA-256 为 `72121b91fc3185805d51ef86aad648156f4207cc28302dcc723432f23f85bcc5`。
- 对模型 14 个文件、GGUF 3 个产物和引擎 55 个文件复核 SHA-256，72/72 相符。[私有证据](https://github.com/nanake-lab/nemu-evidence/blob/e9e72e0860fa6040f6e3b896fb4a07857076382f/P5-13/asset-check.json)，SHA-256 为 `69a698e7e500d63d94f2f5e7737fafc27a6fc34254331b11a0bfa3dccb054f6b`。
- [IRL 冻结输入](../../workers/model-adapter/tests/data/irl-inputs-v1.jsonl) SHA-256 为 `da05c8d65dffc593d22dc4b5560f2601a3790ae736b99b86789c7974a8a7a3bf`。中、英、日各 100 条。每种语言同义改写 12，否定、条件、愿望、引语、用户自报、纠正、歧义指代、新造词各 11。每行有稳定 ID，annotation 留空。输入由 P5-13 生成，未经人工标注，只用于 schema 合格率和延迟。
- MSVC 对正式、无探针、漏注入三种构建均成功，已核对导出函数并从 DLL 读回构建类型。正式载体 SHA-256 为 `163a48c6597e4fd05adbca81c564513ae36d137115755b3e82a79a032d79e6e3`。J01、J05 真实缓存对照、AT13、显存、延迟、300 条 IRL、三语身份、生产回执待验证。

## 载体实现

b11669 进程内 DLL 提供原生 control vector、混合状态清除、重新 prefill、按 token 返回及逐批激活观测；m=0 受约束端点使用同一引擎把调用方给的 JSON schema 编译成 grammar。A/C 与本地认知结果复用生成类型。跨进程探针对象的规范读写仍待[契约所有者确认](../../tasks/requests/P5-13-carrier-probe-object.md)。

[身份问句](../../workers/model-adapter/tests/data/identity-v1.jsonl)三语各 50，SHA-256 为 `8fdedd1c9e013c840e7cba4f2b8fcbabd6d0da14a0ce0c57fac8ef34eda238ca`。实验以名字 Nemu、Windows 桌面伙伴两项事实为 p。非零实验臂使用合成方向，只验证载体；实际方向资格归 P5-31。模式 C 固定同语种草稿，覆盖最终本地改写。

## 验收与限制

| 验收 | 当前证据 |
| --- | --- |
| J01、J05、AT13 | 原生验收分别记录在 [J01](J01/result.json)、[J05](J05/result.json)、[AT13](AT13/result.json)。包内结构测试独立记录。 |
| 本地 IRL | 实测前冻结全部输入。对照 P0-08 原本地 200 条的 schema 192/200、整次 P50 15,647.86 ms、P90 26,168.10 ms。两批语料和模型不同，结果只做描述性比较。[P0-08 对照](../P0-08/decision.md) |
| 同驻显存与延迟 | 三轮各自报告专用与共享峰值之和、二十字首句 P50，同驻负载为舞台和 GPT-SoVITS V5。另记模式 B 加载本地判断端点时的总显存。[窗口脚本](../../workers/model-adapter/acceptance/window.py) |
| 执行回执 | 进程内记录实际模型、tokenizer、信封、hook 位置、覆盖数、注入差值与输出摘要。跨进程存取待核对。[阻塞及处理选项](BLOCKED.md) |

本卡沿用 P0-12 已登记的 llama.cpp b11669（MIT）及 Qwen3.5-4B（Apache-2.0）。Python 运行时使用标准库和仓内生成的 Echo 类型。规格条款、实现和用例的对应关系见[对照表](../../docs/traceability/P5-13.md)。
