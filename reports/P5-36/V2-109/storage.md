# V2-109：真实提交、重开、恢复与续算

复用 `dynamics/storage/tests.rs` 的 65 前缀 C 轨迹、完整私有字段 `to_bits` 断言及每个前缀的全后缀续算。没有新增 C 算法或另一套位比较器。

仓库根执行（现有工具链与临时缓存）：

```powershell
./reports/P5-36/storage/verify-storage.ps1 -Output <新的绝对证据目录>
```

原 kernel 导出真实状态 → core-store 逐前缀在 TemporaryRoot prepare/stage/commit → 关闭全部连接 → Reader 重开并按当前头读取 → 返回读取字节 → 原 kernel 对每个原始状态逐字段比较，并逐事件比较完全相同后缀的状态、ThetaChange、signals、StepBounds。

原始输入、存储返回字节、命令和退出码在 [storage/roundtrip](../storage/roundtrip/)。只把 law_version 绑定为实际准备的合成 LawVersion 对象引用，估计器数值不改。kernel 用原独立固定 law/fuses 验证参数并恢复；store 先检查 bytes 等于写入值，kernel 再检查它等于原真实轨迹的导出值并执行独立内存断言。不声称合成对象是生产资源解析/认证实现。

默认 Rust 测试覆盖已有 0/17/64 三个真实状态；外部验收模式强制恰好 65 个不重复前缀，缺行或只比较 DTO 不能通过。旧数据/非法状态/失败无发布复用同包夹具与故障点，完整工作区结果见主报告。
