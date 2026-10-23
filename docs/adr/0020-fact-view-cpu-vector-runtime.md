# ADR 0020：事实视图的本地 CPU 向量运行时

- 状态：提议；由用户合并本 ADR 即批准。PR 保留 needs-user。
- 任务：P5-04，落实已采纳 ADR 0012 第 7 条，不修改规格、IDL 或阈值。
- 日期：2026-10-23

## 背景

四路召回需要本地多语言向量与 sqlite-vec。模型不得成为新的状态权威，运行时不得访问网络或占用 GPU。AGENTS.md 第 7 节要求对重量级依赖登记 ADR。

## 决策

1. 默认事实视图只有 core-store 只读访问、精确过滤、FTS5 trigram 和 sqlite-vec；CPU 模型实现位于可选 cpu-embedding 编译特性内。用 fastembed 7.3.0（Apache-2.0）、ONNX Runtime 2.0.0-rc.15 Rust binding / ORT CPU（MIT）、tokenizers 0.23.2（Apache-2.0）运行官方 intfloat/multilingual-e5-small。
2. 固定官方修订 614241f622f53c4eeff9890bdc4f31cfecc418b3、MIT 许可、384 维、FP32 基线与同修订官方 model_qint8_avx512_vnni.onnx 候选、attention-mask mean pooling + L2 normalize、512 token 上限、query/passage 前缀、CPU 4 线程。全部模型文件与 tokenizer 的 SHA-256 在 fact-view/models/manifest.json；加载前逐文件核验。文件在 D 盘；运行时只读本地，不自动下载。
3. 查询计划版本 w19-rrf-v1，RRF k=60、每路前 64、固定通道次序和事件 ID 平局次序。不启用可选模型重排。向量是派生索引；所有返回均重读 core-store 核验原始对象。实际查询、模型版本、查询向量位模式、各路排名与最终结果通过生成的 AuditExtension 审计对象交给外层事务持久化；审计对象不作为事实事件索引。
4. 只在 sqlite_vec.rs 集中允许一次已注释的 unsafe FFI 注册，静态链接 sqlite-vec 0.1.9（MIT/Apache-2.0），不加载用户 DLL；其他事实视图模块拒绝 unsafe。
5. 该实现不启用任何正式谱系，不改权威存储格式和表结构，不改变 F/G/C/J。模型升级是显式资源版本更换并重建索引，不能沿用旧向量。

## 备选方案

- Python 子进程：增加进程协议与运维，不采用；本卡不新建手写 IPC 类型。
- 自写模型推理器：增加数值与 tokenizer 错误风险，不采用。
- 云端 embedding：违背本地检索与可重放要求，不采用。

## 后果

CPU 特性增加推理依赖和模型内存；默认编译不加载模型。真实延迟和合成召回数据写在 P5-04 报告，50 ms 仍作为原目标，不以测试夹具冒充多年规模验收。模型选择和重量级依赖由本 ADR 公开登记；用户合并前不作为已批准的生产配置。存储只读扩展按用户专项授权，另在 PR 单列必审范围。

## 量化候选的预注册

FP32 release 初测 9/9 排名第 1、撤回 0/100，但查询中位 87.12 ms，高于原 50 ms 目标。只增加同修订官方 qint8 权重的对照，SHA-256 为 dd476dd0c2514e9b9be83aeb3853fac0763e0bdf4a71645407587d77c48a2d88；仍为 masked mean/L2、512 token、CPU 4 线程。问题、条数、预算、RRF 参数和撤回 100 次不变。ORT 在本机 Ryzen 7 3700X 上选择支持的 CPU 内核，文件名不代表假定本机有 AVX-512。不通过或仍达不到目标时保留失败，不降低阈值。两个权重的版本标识严格区分，索引不能混用。
