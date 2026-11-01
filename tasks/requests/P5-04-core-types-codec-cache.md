# P5-04：内置 schema 描述符的只读 codec 缓存请求

状态：用户已于本会话有条件授权；仅下述两个文件，不扩大范围。

## 证据与原因

最新固定 FP32 release 对照：三语 9/9 正确事实向量与合并第 1，撤回 0/100；完整查询中位 91.13 ms，embedding 多数仅 5–6 ms。原始记录为 reports/P5-04/cpu-eval-fp32-current.log。同修订官方 qint8 在固定排名断言失败，保留失败，不放宽断言，不采用量化候选。

Reader::read 的每次原文核验会调用生成类型的编码/解码；当前 WireMessage 四个默认方法每次都经 Schema::builtin 重新解析编译进二进制的 schema.json。返回多条事实会重复解析同一个不可变描述符。不能以跳过原文核验、减少既定返回条数或放宽 50 ms 目标来避开开销。

## 请求范围（两文件）

1. core/crates/core-types/src/lib.rs：追加私有 OnceLock<Result<Schema,String>> helper，只复用内置描述符。WireMessage 的 canonical_bytes、protobuf_bytes、from_protobuf、from_canonical 使用该只读描述符；所有 encode/decode/normalize/限制检查照常执行。
2. core/crates/core-types/tests/canonical_boundaries.rs：追加缓存与全新描述符在 canonical/protobuf 字节、解码、错误拒绝方面等价的用例，以及并发复用、外部可变 Schema 不影响缓存的用例。

不改 contract.rs、不改生成文件/IDL/黄金向量、不改 Schema::builtin 的现有公开签名和返回独立可变实例的行为，不缓存对象、原文、权限或任何核验结果，不改 core-store 和写路径。

helper 的具体形态：

```rust
fn codec_schema() -> contract::Result<&'static contract::Schema> {
    static BUILTIN: std::sync::OnceLock<contract::Result<contract::Schema>> =
        std::sync::OnceLock::new();
    BUILTIN.get_or_init(contract::Schema::builtin).as_ref().map_err(Clone::clone)
}
```

四个 WireMessage 默认方法内的 contract::Schema::builtin()? 改为 codec_schema()?；其它语义不改。

## 并行范围

2026-10-23 核对 #57（P5-22）正在修改 core-types/src/contract.rs、canonical.rs、generated.rs 和 tests/v2_contract.rs，未修改上述两个文件。但 core-types 目录仍与在做卡重叠，因此本请求也明确请求仅上述两文件的并行例外。#57 合并后 merge main，再跑生成检查、兼容检查、全部黄金向量与完整测试；不向其他执行者发消息。

## 验证

先在 fact-view 自己的范围定向记录描述符构建/复用开销；获授权且确认瓶颈后才改两个文件。字节相同、非法输入仍拒绝、限制仍执行、并发隔离、工作区测试和 CI 全通过后，重跑同一固定 FP32 对照与 100 次撤回。若仍超 50 ms，明确保留阻塞，不扩大范围或降低阈值。PR 单列 codec 缓存范围供审查。

## 用户条件与落实（2026-10-23）

- 先确认局部方案：public Schema 已有 encode/decode 接口；但 Reader::read 没有编译解码器参数，内部 ObjectRef 编码、对象核验和 typed 解码分别调用默认 codec。fact-view 持有一个 Schema 不能注入这些内部调用。100 次编码的定向对照：新建描述符 62.58 ms，复用描述符 0.16 ms，原 WireMessage 路径 60.14 ms。
- 仅缓存不可变内置描述符，OnceLock 一次性构建，键为类型标签、支持的 schema_version=1、生成 schema.json 的 SHA-256。所有输入仍经过原编码/解码和 normalize，未知版本不作为新缓存条目，也不会改变拒绝顺序。
- 冷/热缓存覆盖全部 396 条合法、409 条非法黄金向量，与全新描述符的结果、字节、摘要、Err(String) 精确相同；另用独立子进程验证公开 WireMessage 首次/后续调用等价、外部可变 Schema 不能影响缓存，并测试并发和预算拒绝。
- 不改 contract.rs、IDL、生成文件、黄金字节、Python/TypeScript codec、存储或写路径。和 #57 并行仅上述两个未重叠文件；后合并者 merge 最新 main 并跑全部契约检查。
- #65 PR 单列 core-types 扩展，附查询优化前/后耗时，纳入必审范围。首次对照：FP32 完整查询中位 91.13 → 42.16 ms，最大 46.54 ms；固定三语 9/9 第 1、撤回 0/100 不变。最终 CI 与 main 同步后仍需重跑。
