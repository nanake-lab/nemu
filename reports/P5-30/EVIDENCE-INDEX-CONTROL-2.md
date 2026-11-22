# P5-30 L0-control-2 证据索引

第一版对照 / 非正式结论。原始合成输入、输出和日志保存在私有 `nanake-lab/nemu-evidence`，公共仓库保留报告及身份索引。

## 版本与冻结顺序

- 执行源码：4a063526fe62e45f912773ba4a72b81664ffd5fd。
- main：d1213c8666efd504b6f6cd0488716419f4077406，包含 #194。
- 私仓冻结：5f5be17202ce5a07d56e71c012d8ebf1a508b0d4。
- 远端确认及冻结 Git-blob 索引：83905517e8f3e2de4a5fe7022097891f733d07dc。
- 主仓执行前说明：0ce6c084f62bf128ccc6984a7d049daa130f9fb1。
- 本机启动 UTC：2026-11-15T06:15:50.381007+00:00；主单元首个 BEGIN 为 06:15:52.255092，最后 END 为 09:45:50.334668。均在上述推送/远端核对之后；本机时钟不是独立服务器时间证据。

## 完整结果身份

私仓目录 `P5-30/v0.2/L0-control-2/`。

- 完整结果提交：[acd1e5c50c28cbf1fde902dac93ecdf528b07058](https://github.com/nanake-lab/nemu-evidence/tree/acd1e5c50c28cbf1fde902dac93ecdf528b07058/P5-30/v0.2/L0-control-2)。
- Git-blob 核验提交：[a1fdd75d560be63c2102752fabef5bd6d8501d17](https://github.com/nanake-lab/nemu-evidence/blob/a1fdd75d560be63c2102752fabef5bd6d8501d17/P5-30/v0.2/L0-control-2/GIT-BLOB-VERIFICATION.json)。
- `EVIDENCE-INDEX.json`：索引 1681 个文件、2657174888 字节（不含索引自身及随后追加的核验文件）；索引自身 SHA-256 为 349b2ed0e6d0557d93eb9c060951013b11f91ade6a39af63122fb91c3b205700。
- `GIT-BLOB-VERIFICATION.json`：逐个从完整结果提交读取 blob，核对索引及索引自身的长度与 SHA-256，附每个 Git 对象 ID；同时核对远端分支与旧 control-1 未改变。
- `FREEZE.json` / `REMOTE-CONFIRMATION.json`：运行前输入、执行器、二进制及其 Git 字节身份；执行后再次核对全部冻结文件。
- `run01/`：180 条主单元的事件轨迹、日探针、检查点和 result；12 个 Q2 续接输出及全部 1512 对配对记录；冻结分析的完整汇总。
- `FINISH.json`、`EXECUTION-STATUS.md`、`validation/control2-execution-run01.log`：结束观察、状态与主单元绝对 BEGIN/END。
- `REPORT-STATS.json` / `REPORT-PROJECTION.json`：只读报告整理；其脚本在 validation，明确为运行后整理，不属于预冻结执行器。
- `validation/CI-SNAPSHOT.json`：0ce6c084 的 CI 状态，Windows 实际检查 skipped 不算通过。

目录在写入冻结输入前启用 `** -text`。本版本索引绑定存储字节，逐个 Git blob 核验；没有以 LF/CRLF 语义等同代替字节一致。

执行后 Git 核验通过：1682 个 blob、2657447114 字节（含索引自身，不含随后追加的核验记录）。核验记录自身 385354 字节，SHA-256 814a2c2ffc31ba67758bed8bdb771fa6c3f8d750df4213c8b5da1ed823ba68fd；该记录已另提交推送。

## 大汇总的两种身份

| 对象 | 字节数 | SHA-256 |
| --- | --- | --- |
| 逻辑原件 run01/summary.json | 606099125 | e932bd057bff72cea0fe170735fb0323aa09778232d59d1aecffbfec94c2a952 |
| Git 保存的 run01/summary.json.gz | 27311960 | 92645b5aaa1f7189056be54a412192edf40dc34033f1048c2387a70609985cb7 |

`ARCHIVES.json` 记录原件到归档的映射。gzip 固定 mtime=0；归档后解压并重新计算逻辑原件 SHA-256，核对通过。本机未压缩原件保留，用 .gitignore 排除超限大文件；没有删除或改写原始结果。其他文件按索引原字节保存。

## control-1 保留与追加勘误

旧结果 b9195e22c3b3ba28ead552c29c7de28453e6d918 的 `P5-30/v0.2/L0-control-1/` 保持不变。勘误提交 e40095fac981f1ee059fc736f6a1e226447b68d1 位于独立 `L0-control-1-errata/`：258 个旧 Git blob 的索引，以及本机 CRLF FREEZE 原件的另存副本。

Q2 正向守约分支原缺陷判断已撤销；旧输入 F01 和旧 not_run 不改。完整说明见 [追加勘误](CONTROL-2-REVIEW-RESPONSE.md)，新结果见 [control-2 报告](CONTROL-2-REPORT.md)。
