# 本轮存储恢复验证

命令、时间与返回码见 [commands.json](commands.json)，结论见 [result.json](result.json)。三个原始日志分别为 [导出](export.log)、[实际提交/关闭重开/读取](store.log)、[逐位恢复及全后缀续算](restore-continue.log)。

本轮重新生成的 `actual-states.json` 和 `stored-states.json` 与已入库原始数据逐字节一致；不再重复提交相同大文件，本机新文件仍保留。对照哈希与路径见 [state-evidence-reuse.json](../state-evidence-reuse.json)，可读原始数据为 [actual-states.json](../../review-fix/storage-roundtrip/actual-states.json) 和 [stored-states.json](../../review-fix/storage-roundtrip/stored-states.json)。
