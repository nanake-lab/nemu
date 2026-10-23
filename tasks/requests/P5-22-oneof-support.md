# P5-22 配套范围授权

用户在 2026-10-22 的本聊天中授权必要的配套修改，并要求工作放 D 盘。

范围：core/contracts 的 oneof 解析/生成/夹具/兼容检查工具、Python/TypeScript 模板、core-types 通用校验和 CanonicalV1，以及生成产物。仅实现 ADR 0007 §6 已规定的编码，不改变其他编码规则；所有现存向量逐字节保持；至少三个 oneof 合法向量，非法覆盖缺失、多选、未知字段号、尾随字节；compat 检查成员变化和号复用；报告单列配套修改，与契约一起交 Claude 交叉审查。

Claude P5-01 回溯审查关联：[PR #4 comment](https://github.com/nanake-lab/nemu/pull/4#issuecomment-6332237760)。按本卡 oneof 的需要落实 C1(b,c) 的文本解析与字段升序断言；其他问题单列后续，不改变本卡以外的编码规则。
