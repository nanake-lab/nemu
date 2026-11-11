# P5-42 证据索引

原始证据：[私有 nanake-lab/nemu-evidence 提交 `16154210155c9fbaaf2b7d49202867a3056eb5c5`](https://github.com/nanake-lab/nemu-evidence/tree/16154210155c9fbaaf2b7d49202867a3056eb5c5/P5-42)。逐文件仓库路径、SHA-256、提交号、字节数见 [EVIDENCE-INDEX.csv](EVIDENCE-INDEX.csv)。推送后已核对远端提交，且逐文件比较 Git blob 与本机原件一致；目录关闭文本换行转换。

- `2026-11-11-local/`：全部本机原始日志（含失败/中断）、未发布的早期向量后缀、运行辅助脚本、最终源码 raw SHA-256 / Git blob 清单。
- `ubuntu-43044637186/`：官方运行记录、完整日志、实际 checkout / 双亲核对、上传产物。
- `windows-43047851482/`：手动全量官方记录、完整日志、实际 checkout（见 checkout-verified.json）、全部上传产物。
- `draft-gates/`：草稿自动 Windows 门禁失败 / 自托管跳过的原始记录；不是代码失败，也不是 Windows 通过证据。

以 `*-final` 和两次成功 CI 为最终验证。早期检查针对当时未提交实现，不能只用辅助脚本记录的结束时 HEAD 冒充实际被测提交。`workspace-test-01` 的编译源早于最终 β=0 身体修正，并因 main CI 抢占资源中断，明确不计为完整工作区通过。最终源码由 `source-manifest.json` 及 CI checkout 绑定；其后仅报告收尾。

旧 616 合法 / 860 非法向量保留，新消息增加 30 / 41；每种新消息至少 3 合法、3 非法。三语言资源/回执检查与 P5-12 数值往返只使用合成数据；真实载体、生产原子发布与谱系 SYS17 为 not_run。
