# P5-02 审查修订一

## 结果

修复、本机完整复验与本轮 Ubuntu/Windows CI 均通过；提交独立复审，未合并。此报告取代初次报告的当前状态；原始日志和结论保留为历史证据。本实现会话不签发审查通过结论。

审查基线 `6209401822a1a6cce43708447c6f6d180e93c798`；[阻塞意见](https://github.com/nanake-lab/nemu/pull/58#issuecomment-6341822078)。已先后 merge `origin/main` 的 `47ee2845c4731969242fe0a809ccc7637e1cf55c` 与 `1f7d4734f32a399e9f867427864ddf87691755e9`；后者仅更新其他卡及 P5-13 的依赖，未改变 P5-02 要求。未 rebase、未强推。

## 问题与修正

| 审查项 | 改动 | 证据 |
| --- | --- | --- |
| A1 | 抽出只读完整核验，Reader 与 Store 共用；缺失 L、混合 head、对象损坏拒绝打开 | 本机通过；[AT02](AT02/result.json) |
| B1 | 删除仅凭环境变量路径打开私有 Store 的入口；父进程新建 TemporaryRoot，随机一次性 permit 经 stdin 与根目录文件双重匹配，验证和消费后才开锁/SQL | 本机通过；[I29](I29/result.json) 的沙箱负例 |
| B2 | 先捕获回执，再固定 SQL 快照，在同一快照核对完整性；测试在 capture 后、SQL 快照后分别追加 | 本机通过；[AT04](AT04/result.json)，保留真正旧备份拒绝 |
| C1 | 按操作/tag/第 N 次写入选择真实进程退出；D、提交、回执及补写、R 子对象/索引、复用、清单、初始化、GC 分列 | 本机通过；[完整矩阵](final-test-manifest.json) |
| C2 | 三种谱系都写入可区分记录；各自正向读取、六向对象/打开拒绝，LIFE 事实视图没有 REHEARSAL 记录 | 本机通过；[I29](I29/result.json)、[AC2-22](AC2-22/result.json) |
| D1 | 保留临时属性清除，加同目录 MoveFileExW WRITE_THROUGH；列明 FlushFileBuffers、目录句柄和证据边界 | 本机 NTFS 原语测试及本轮 Windows CI 通过；README |
| D2 | merge main 后重跑检查，不沿用旧提交 CI 代替 | 本机完整复验通过；结束后再 fetch，main 仍为 1f7d473 且已在本分支中 |

## 最终本机验收

冻结源码提交：`1e1cc584dba4abaa91be11f2296195a2f320e303`。检查源文件树未变；后续仅补证据。按项目 routine_reporter 角色机械执行，主代理核对退出码、原始输出、逐项场景和来源指纹。重负载独占，未使用 GPU；测试全部使用父进程新建 TemporaryRoot。

| 检查 | 结果 | 原始证据 |
| --- | --- | --- |
| cargo fmt --all -- --check | exit 0 | ../logs/13b-review-fmt.log 与 .json |
| cargo clippy --workspace --all-targets --locked -- -D warnings | exit 0 | ../logs/14-review-clippy.log 与 .json |
| cargo test --workspace --locked -- --test-threads=1 --nocapture | exit 0；85 passed、0 failed，其中 core-store 31 项运行测试和 2 项 compile-fail doctest | ../logs/16-review-final-workspace.log 与 .json |
| 日志与验收映射核对 | 13 组编号；82 场景、100 次实际 exit 86 注入，全部匹配预期 | summarize.py、final-test-manifest.json，各编号/result.json |

完整测试用时约 15m57s（2026-10-22 22:11:51–22:27:49 +08）。原始日志 SHA-256 为 `43d72bdfb78581381b23ebcd121841a891973ebc6e4d4813ff48baabb8f8948a`；LF/UTF-8 规范化后为 `22ab190bf118456ca5f047b2aa88223c5d9c2a498874593da6c6850fd3e13f78`。Git 会规范化文本换行，因此复核脚本核对后者，同时保留采集时的原始字节摘要。命令退出码直接采集，不从日志推断。

82 个场景分为：提交 6、D 入口 7、回执发布/补写 36、R 子对象/索引与第 N 份事务清单 12、对象复用 4、初始化 13、GC 4。回执修复有 18 次先行 SQL 提交后崩溃，因此总实际退出次数为 100。前 65 项是 X03 提交相关矩阵；初始化和 GC 分开列证，不扩大 X03 定义。

## CI 与最终状态

两平台检查的 head 均为 `670c760d2f518703eb2b3a4678b30c4ee4798837`，与实现提交 `1e1cc58` 的 core/ 和相关 CI 文件树一致。此后只追加报告、原始日志和机器可读证据，不改源码。再次 fetch 后最新 main 仍为 `1f7d473`，已包含在任务分支中。

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| Ubuntu CI | run 与 job 均 completed/success；Rust 85 项通过，含工作区静态检查和契约一致性 | [run 39796228753](https://github.com/nanake-lab/nemu/actions/runs/39796228753)；../logs/17-review-ci-39796228753.log |
| Windows CI | run 与 job 均 completed/success；31 + 2 项 W02 测试；82 场景、100 次退出与本机逐项匹配；桌面/隔离/视觉及证据检查通过 | [run 39796248748](https://github.com/nanake-lab/nemu/actions/runs/39796248748)；windows-39796248748/；../logs/17-review-ci-39796248748-direct-job-raw.log |
| 证据核对 | 源提交、完整日志和文件指纹已保存 | [ci-evidence.json](ci-evidence.json)；../logs/17b-review-ci-final-status.json |

Windows job 运行约 18m53s，保留原 20 分钟上限，没有放宽超时或阈值。早期 GitHub API 的 run/job 终态曾不同步；旧快照原样保留，最终快照确认两层均 success。采集遇到一次网络 EOF、本地查询包装 ParserError，以及 run 尚未完成导致日志读取拒绝；直接 job 日志首次因终端转义序列被 CLI 拒绝，随后只重定向原始字节到文件成功，未在终端解释控制序列。这些是采集诊断，不是测试失败，所有尝试均保留在 logs/17*。没有为此重跑 CI。

## 证据边界与更正

- 原报告“所有测试只创建 TemporaryRoot”的无条件表述不够精确：父测试确实创建临时根，但旧 worker 可由外部环境变量指定路径，且取得锁/打开数据库后才检查 temporary 标记。B1 是真实入口缺口；现已改为先验证一次性沙箱凭据。复现及修复测试均只使用新建临时目录，没有打开用户排练库或正式库。
- 原测试名 every_durability_boundary 及完整 X03 覆盖的表述过宽。新矩阵逐项列明注入操作、对象类型、命中次数与恢复结果；初始化和 GC 单列，不据有限注入推断所有 I/O、物理断电或固件故障。
- SQL commit 是状态提交点，完整回执同步后才返回成功。SQL 已提交而尚未 ACK 可以恢复为新状态；补回执再次崩溃仍必须可验证。
- 参考 [MoveFileExW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw)、[FlushFileBuffers](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-flushfilebuffers)、[CreateFileW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew)。名称可见性依赖本机 NTFS 同目录改名，数据同步依赖文件 flush，发布持久化依赖明确的 WRITE_THROUGH 与后续目录 flush。目录句柄标志本身不是持久性证明；仍假设 OS/文件系统/设备遵守同步语义，没有独立验证突然断电下所有命名空间前提。
- 测试沙箱凭据防止误设环境变量和无上下文启动；不宣称抵御同权限恶意进程伪造全部文件。正式库权限隔离仍由其所属卡实现。

## 新增依赖与范围

无新增依赖、IDL、格式版本、ADR 或阈值变更。存储格式仍为 1，CanonicalV1 与后继类型不变。源码只改 core-store；规格对照只追加本轮说明。

## 下一步

最终证据提交后转为可审查，在 PR 中逐项回复阻塞意见并请求复审；请 Claude 交叉审查，通过前不合并。当前 main 的 tasks/P6-presence.md 追加记录允许本次 Codex 独立审查，且诞生前仍需 Claude 补审崩溃安全与 I29；实现会话不自行签发审查结论。不认领下一张卡。

## 中间检查记录

- `11-review-check.log`：首次编译 E0716（测试中借用临时 tag 字符串），已改为具名绑定；保留原日志。
- `12-review-workspace-tests.log`：中间版 core-store 29 passed、1 failed；R 黄金夹具为空，新增用例错误取第一个元素。已改为显式构造两个 TendencyV2，并补非空 R 发布/重开/GC 的正向用例。该版 Windows 包装还未补回临时属性清除，不作为最终证据。
- `13-review-fmt.log`：采集包装因空输出未建文件而失败，cargo 退出码未保存，记为未知；`13b-review-fmt.log/.json` 为随后明确记录 exit 0 的检查。
- `14-review-clippy.log/.json`：工作区全部 target，`-D warnings`，exit 0。
- `15-review-targeted.log/.json`：R/manifest/复用 16 个真实退出组合通过（单个矩阵测试）；不能代替最终完整工作区运行。
