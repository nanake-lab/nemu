# P5-30 L0-control-1 私有证据索引

第一版对照 / 非正式结论；执行包有效性 failed。原始结果只作诊断，不能引用其局部 passed 作为预注册通过。

仓库：`nanake-lab/nemu-evidence`，分支 `evidence/P5-30-control`。

- 冻结输入提交：`7340ee79f3e13a0058343cab9fcbaaa8d105c8c7`。
- 结果与失败提交：`b9195e22c3b3ba28ead552c29c7de28453e6d918`。
- [全部证据](https://github.com/nanake-lab/nemu-evidence/tree/b9195e22c3b3ba28ead552c29c7de28453e6d918/P5-30/v0.2/L0-control-1)。
- `EVIDENCE-SHA256.json`：258 个文件的 SHA-256 与长度；其自身由 Git 提交绑定。
- `FREEZE.json`、`package/sha256.json`：源码、优化二进制、law/genesis/参数、200 单元清单、事件/探针/候选/μ。
- `VALIDITY-FAILURE.json`：102 条受影响事件表、各自首个原始见证、停止调度说明。
- `EXECUTION-STATUS.json`：6 完成、174 未启动、20 预登记未运行，逐条状态与原因。
- `run01/`：六条完整压缩事件和每日探针、四检查点、结果、原始日志；`summary.json` 是冻结分析器的只读诊断。
- `validation/`：完整 release 工作区测试、最终机械测试/静态检查、原始失败与 debug 中止说明。
- `experiment-world.exe`：实际执行的 Windows release 二进制；SHA-256 与冻结记录一致。

总文件内容 165,915,878 字节（不含索引自身）；没有上传用户历史或正式存储。不得覆盖失败或补写成全部完成。


## 2026-11-15 追加勘误与另版执行

上文为 L0-control-1 历史记录，原文保留。Q2 守约正向分支是预注册 v0.2 §6 的要求；撤销“注册外事件/适配缺陷”的判断及删除该分支的建议。control-1 的 Q2 仍为 not_run，人物映射 F01 仍成立。

旧 FREEZE.json 本机 CRLF 原件为 872 字节（39af51fbea8e762c055b9774251adfaafaee481a26c84450ac9208bf60d59170），旧 Git LF blob 为 857 字节（0ce73a71af2c5dc01317da6108cab48e10e47f0320e5a865b5067a2d57e1bb8c）。旧索引绑定本机字节，追加 Git-blob 索引绑定旧提交；两者不混用。证据见 [审查回复与勘误](CONTROL-2-REVIEW-RESPONSE.md)。

用户已批准 L0-control-2 修正人物规则、补完整输入校验，绑定含 #194 的 main，重新冻结推送后运行。新版见 [执行记录](CONTROL-2-EXECUTION.md) 和 [独立报告](CONTROL-2-REPORT.md)，不替换旧版失败证据。
