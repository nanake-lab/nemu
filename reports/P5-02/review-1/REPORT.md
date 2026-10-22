# P5-02 审查修订一

## 结果

修复与补验进行中；PR #58 保持草稿，未合并。此报告取代初次报告中“全部验收完成”的当前状态；原始日志和结论保留为历史证据。

审查基线 `6209401822a1a6cce43708447c6f6d180e93c798`；[阻塞意见](https://github.com/nanake-lab/nemu/pull/58#issuecomment-6341822078)。已先后 merge `origin/main` 的 `47ee2845c4731969242fe0a809ccc7637e1cf55c` 与 `1f7d4734f32a399e9f867427864ddf87691755e9`；后者仅更新其他卡及 P5-13 的依赖，未改变 P5-02 要求。未 rebase、未强推。

## 问题与修正

| 审查项 | 改动 | 证据 |
| --- | --- | --- |
| A1 | 抽出只读完整核验，Reader 与 Store 共用；缺失 L、混合 head、对象损坏拒绝打开 | 待最终运行 |
| B1 | 删除仅凭环境变量路径打开私有 Store 的入口；父进程新建 TemporaryRoot，随机一次性 permit 经 stdin 与根目录文件双重匹配，验证和消费后才开锁/SQL | 待最终运行 |
| B2 | 先捕获回执，再固定 SQL 快照，在同一快照核对完整性；测试在 capture 后、SQL 快照后分别追加 | 待最终运行 |
| C1 | 按操作/tag/第 N 次写入选择真实进程退出；D、提交、回执及补写、R 子对象/索引、复用、清单、初始化、GC 分列 | 待最终运行 |
| C2 | 三种谱系都写入可区分记录；各自正向读取、六向对象/打开拒绝，LIFE 事实视图没有 REHEARSAL 记录 | 待最终运行 |
| D1 | 保留临时属性清除，加同目录 MoveFileExW WRITE_THROUGH；列明 FlushFileBuffers、目录句柄和证据边界 | README；待最终 Windows CI |
| D2 | merge main 后重跑检查，不沿用旧提交 CI 代替 | 待最终运行 |

## 证据边界与更正

- 原报告“所有测试只创建 TemporaryRoot”的无条件表述不够精确：父测试确实创建临时根，但旧 worker 可由外部环境变量指定路径，且取得锁/打开数据库后才检查 temporary 标记。B1 是真实入口缺口；现已改为先验证一次性沙箱凭据。复现及修复测试均只使用新建临时目录，没有打开用户排练库或正式库。
- 原测试名 every_durability_boundary 及完整 X03 覆盖的表述过宽。新矩阵逐项列明注入操作、对象类型、命中次数与恢复结果；初始化和 GC 单列，不据有限注入推断所有 I/O、物理断电或固件故障。
- SQL commit 是状态提交点，完整回执同步后才返回成功。SQL 已提交而尚未 ACK 可以恢复为新状态；补回执再次崩溃仍必须可验证。
- 参考 [MoveFileExW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw)、[FlushFileBuffers](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-flushfilebuffers)、[CreateFileW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew)。名称可见性依赖本机 NTFS 同目录改名，数据同步依赖文件 flush，发布持久化依赖明确的 WRITE_THROUGH 与后续目录 flush。目录句柄标志本身不是持久性证明；仍假设 OS/文件系统/设备遵守同步语义，没有独立验证突然断电下所有命名空间前提。
- 测试沙箱凭据防止误设环境变量和无上下文启动；不宣称抵御同权限恶意进程伪造全部文件。正式库权限隔离仍由其所属卡实现。

## 新增依赖与范围

无新增依赖、IDL、格式版本、ADR 或阈值变更。存储格式仍为 1，CanonicalV1 与后继类型不变。源码只改 core-store；规格对照只追加本轮说明。

## 下一步

补齐最终本机检查与 Ubuntu/Windows CI，回复原审查评论并请求复审；通过前不合并。当前 main 的 tasks/P6-presence.md 追加记录允许本次 Codex 独立审查，且诞生前仍需 Claude 补审崩溃安全与 I29；实现会话不自行签发审查结论。

## 中间检查记录

- `11-review-check.log`：首次编译 E0716（测试中借用临时 tag 字符串），已改为具名绑定；保留原日志。
- `12-review-workspace-tests.log`：中间版 core-store 29 passed、1 failed；R 黄金夹具为空，新增用例错误取第一个元素。已改为显式构造两个 TendencyV2，并补非空 R 发布/重开/GC 的正向用例。该版 Windows 包装还未补回临时属性清除，不作为最终证据。
- `13-review-fmt.log`：采集包装因空输出未建文件而失败，cargo 退出码未保存，记为未知；`13b-review-fmt.log/.json` 为随后明确记录 exit 0 的检查。
- `14-review-clippy.log/.json`：工作区全部 target，`-D warnings`，exit 0。
- `15-review-targeted.log/.json`：R/manifest/复用 16 个真实退出组合通过（单个矩阵测试）；不能代替最终完整工作区运行。
