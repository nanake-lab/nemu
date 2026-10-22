# P5-02 单写事务与对象存储（W02）— 报告

## 结果

完成：实现、同步 main 后的本机验收和 Ubuntu/Windows CI 均通过。源提交 `5cc3b540f3b7b33b611a4c590cac91c9406df80a`；随后仅追加报告与证据。PR #58 提交交叉审查，尚未合并。
本卡涉及账本与存储格式，必须由 Claude 交叉审查；本代理不合并。

## 交付物

- `core/crates/core-store/`：单写 SQLite、CanonicalV1 内容寻址对象、R 倾向分段、
  完整绑定的暂存/提交能力、谱系类型、证据/账本索引、恢复核验、故障注入和临时库测试。
- `core/crates/core-store/README.md`：存储格式、API 边界、提交协议与恢复凭据说明。
- `.github/workflows/windows.yml`：只追加 `core/**` 路径与串行 W02 检查、独立证据上传。
- `docs/traceability/P5-02.md`：逐条规格、实现及测试映射。
- `tasks/requests/P5-02-acceptance-registration.md`：W16 中央验收入口登记请求；
  测试留在本卡授权的 crate 范围。

## 验收

最终本机检查基于上述源提交；`git fetch origin` 后 main 仍为 `91b31ed2d30aadd8cc1cf8086dd0aad8ffe03821`，merge 返回 Already up to date。

| 最终命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| cargo fmt --all -- --check | exit 0 | logs/09-final-fmt.log |
| cargo clippy --workspace --all-targets --locked -- -D warnings | exit 0 | logs/08-clippy.log |
| cargo test --workspace --locked -- --test-threads=1 --nocapture | exit 0；77 passed，0 failed；约 181 秒 | logs/10-final-workspace-tests.log |
| W02 运行测试 / 编译拒绝测试 | 23 / 2 passed | final-test-manifest.json |
| Ubuntu CI | success | [Ubuntu run](https://github.com/nanake-lab/nemu/actions/runs/39777111967) |
| Windows CI（草稿阶段手动触发） | success；含 W02 NTFS 检查 | [Windows run](https://github.com/nanake-lab/nemu/actions/runs/39777926877) |

13 组验收编号各有 `result.json`；manifest 保存源文件树指纹、规范换行后的日志 SHA-256、完整命令和套件计数。`summarize.py` 只核对日志并生成证据，不替代测试。测试环境为 rustc 1.99.0 / cargo 1.99.0、Windows NTFS、SQLite 3.53.2。

前期原始证据已保留：

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| 首次 cargo check | 失败；SQLite u64 读取需显式范围转换，已修正 | logs/01-cargo-check.log |
| 第一次存储测试 | 15 passed | logs/03-store-tests.log |
| 扩充恢复、绑定、P5-32 索引测试 | 21 passed；10 个真实进程退出点 | logs/04-store-tests.log |
| 工作区 fmt | passed | logs/05-fmt.log |
| 首次工作区 Clippy | 失败；collapsible_if，已修正 | logs/06-clippy.log |
| 修正后工作区 Clippy，包含恢复核验 | passed | logs/08-clippy.log |

## 规格对照

详见 [P5-02 对照表](../../docs/traceability/P5-02.md)。涉及 I01/I02/I03/I14/I18/I29，
按类型能力、运行时核对、只追加约束和恢复核验分别落实。未新增 IDL 消息，未改
CanonicalV1、黄金向量、原规格、验收阈值或差异登记。

R 使用 P5-32 的 `RSnapshotV2` / `TendencyV2`，证据组使用 `EvidenceGroupV2`；
事务和绑定使用 P5-21 后继类型。R 的物理索引重组后必须逐字节等于生成的规范编码。
θ、L 条目、提交快照均为生成类型的 CanonicalV1 对象。

## 新增依赖

许可证来自已解析、锁定版本的 Cargo 包元数据；SQLite 源码版本为 **3.53.2**。

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| rusqlite | 0.40.2 | MIT | SQLite 连接与事务，bundled / limits |
| libsqlite3-sys（传递） | 0.38.2 | MIT；SQLite 源码公共领域 | 固定 SQLite 构建，含 WAL-reset 修复 |
| serde_json（已有工作区版本） | 1.0.154 | MIT OR Apache-2.0 | 生成契约反射、规范编码 |
| tempfile | 3.27.0 | MIT OR Apache-2.0 | 同目录原子文件准备与隔离临时测试根 |
| tracing | 0.1.44 | MIT | 仅记录位置与提交状态，不记录正文 |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 | Windows 本机 NTFS 卷查询 |

两次 CI 的完整原始日志见 `logs/ci-39777111967.log`、`logs/ci-39777926877.log`，来源、摘要与结论见 `ci-evidence.json`。

完整解析依赖固定在 `core/Cargo.lock`。没有新增 GPL/AGPL 运行时依赖。
生产源文件均小于 400 行。Win32 卷查询的 unsafe 只位于 platform.rs，逐处说明
指针/缓冲区前提；其余模块使用 crate 级 deny(unsafe_code)。

## 待用户确认

- 没有新增用户决策项：无新 ADR、候选定稿、契约变更或验收阈值变更。
- Claude 交叉审查是本卡的必需审查门，尚未取得审查结论；通过前不合并。

## 证据边界

- 所有测试只创建 `TemporaryRoot`；包括 LIFE 的测试夹具。没有打开用户排练库、
  正式库，没有执行生产 birth/growth，没有使用 GPU。
- X03 使用真实子进程退出、跳过析构，再重新打开临时库；另有模拟存储 I/O 失败
  和真实 `SQLITE_FULL`。没有实施物理断电或磁盘固件实验。
- AT04 覆盖双写拒绝、复制库不能冒充原分支、旧数据库在原位置恢复后拒绝继续。
  冗余持久化凭据放在库目录旁，普通打开不会重置。W14 / P5-15 仍负责恢复尾部、
  登记有损分支及外部备份来源核验；整台设备连独立凭据一起回退无法仅靠本地软件检测。
- W02 检查 runtime 提供的绑定并原子保存结果，不计算或证明 Φ/A/M/F/G/C 的数学
  正确性。结构夹具不构成成长、人格或行为有效性的证据。
- 操作系统 worker 隔离归启动器/网关；本卡提供单写锁、不可伪造写能力和谱系检查，
  不宣称仅靠 Rust API 可以阻止同权限恶意进程直接篡改文件。

## 后续建议（不在本任务范围内）

- P5-15 在备份/恢复实现中接入独立凭据与显式恢复流程。
- P5-18 / W16 登记本卡测试与编号；其他卡的系统/行为验收保持各自范围。

## 下一步

全部验收通过，转为可审查并在 PR 中请求 Claude 交叉审查。等待其审查结论；本轮不合并、不认领下一张卡。
