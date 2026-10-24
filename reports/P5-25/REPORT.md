# P5-25 动力学 C：θ 的学习与回应回路 — 报告

## 当前状态

**2026-10-24 本轮更新：** 环境仍在，已逐步定位1 ULP首因；未修复、未放宽断言。精确整数舍入证明Python平台`**2`的结果偏离最近值，Rust显式乘法在该步符合ADR0019。21项运算和sqrt交叉输入已在debug/优化构建核对；直接powf会随优化级别改变，未用于生产。待用户明确是否允许固定次序的Python规范参考。详见 [本轮定位报告](rounding-20261024/REPORT.md)。

**部分完成，有阻塞，保持草稿 PR #77。** C 的候选实现、10项定向测试、原THETA输入与证据已提交供审查。定向测试9通过/1失败；THETA-1跨语言逐位oracle的σ有1 ULP差异，不改阈值、不删断言。THETA-2固定ρ与自适应检测器的反馈测试通过。全工作区测试另遇执行会话丢失，结果不完整。详见 [BLOCKED.md](BLOCKED.md)。未发布新规律，未接入正式谱系，不合并或取下一卡。

### 云端试点记录

- **2026-10-24 收尾更正：** 昨夜上传执行失效，远程PR仍停在预检提交，并非整夜正常计算。核验47个对象已存在，2.4 MB的theta1.bin对象缺失导致未生成提交。保留原始逻辑字节与哈希，将该文件无损拆为19片，测试仅调整读取拼接，未改实现、断言或重跑失败验收。详见 `evidence/upload-recovery.json`、`evidence/tests/theta1-storage.json`。

- 基线及发布前复核 main：`0b5412e95caab447c70148c473e544dbab28c57c`；未发现更新。真实远程分支 `task/P5-25-dynamics`，草稿 [PR #77](https://github.com/nanake-lab/nemu/pull/77)。
- 能力预检先只读：已授权GitHub连接器可读/写该仓库；随后按用户条件开工。终端直接Git无认证，通过连接器按固定提交取得全部非reports文件与reports下代码，共1,761文件，逐文件校验Git blob SHA-1及字节数。旧报告的非代码证据未整体下载；这不是含完整历史对象的clone，content gitlink未展开。清单见 `evidence/environment/source-manifest.json`。
- 发布同样走已有授权GitHub Git数据接口，以真实main及任务分支提交为父节点和base tree保留未下载文件；没有把本地合成snapshot历史推上远程。未请求/读取/注入token或密钥，未绕过权限。
- Linux；Node **24.19.0**、pnpm **12.11.2**、uv **0.12.15**、Python **3.12.14**、Rust **1.99.0**（含clippy/rustfmt）。pnpm/uv/Rust位于用户目录。`pnpm install --frozen-lockfile`、`uv python install 3.12`、`uv sync --locked --project core/contracts`、`rustup show`、`cargo fetch --locked`均已成功；原始安装输出全部在 `evidence/environment/`。
- 一次源码连接中断在同接口重试恢复，保留原失败记录。首次cargo从仓库根运行无法选toolchain，未运行测试；改为CI同样的core工作目录后开始测试。未换版本。
- 全工作区运行日志结束于既有core-store崩溃测试，session后来不可用；未得到终态，原因不明，不以“本地可编译”声称全CI通过。`execution-status.json`记录这一限制。
- 用户本次豁免D盘/Windows runner本机条款。未初始化content、未使用密钥、未打开用户数据或任何排练/正式谱系存储；测试只用固定种子合成数据和既有测试创建的临时存储。
- 已读PR #73 head `25e8474958c7ccd9d0d37266a881c8df733f7058` 的θ提醒：自己的连续反应似然、独立反馈稳定性检验、保留变化率下限。采用临时连续高斯均值CUSUM候选，未搬用R的二元后验。

## 交付物

- `core/crates/core-kernel/src/dynamics/`：纯C暂存、完整临时估计器状态、连续反应检测器、b/σ/g/τ/λ递推、原子hold式跳变/慢漂信号、显式数值故障与10项单元/性质测试；`lib.rs`仅追加模块登记。
- [METHODS.md](METHODS.md)：参考选择、数值次序、输入信任边界、时长/熔断候选及复现命令。
- [临时契约请求](../../tasks/requests/P5-25-dynamics.md)：正式估计器资源、外层L事务、P5-26及统一验收目录接入；`core/contracts/`和既有黄金向量未改。
- `generate_fixtures.py`、`diagnose_theta1.py`及`evidence/`：原种子72,822个THETA-1事件、6,000个THETA-2噪声样本、Python原结果、全部失败/成功日志与哈希。

## 验收

| 命令或检查 | 结果 | 证据/边界 |
| --- | --- | --- |
| 指定环境安装和锁文件拉取 | passed | evidence/environment/ |
| 原Python THETA-1/2 | passed，零违例 | evidence/tests/original-and-fixtures.json，原脚本未改 |
| Rust THETA-1 移植与附加逐位oracle | **failed** | dynamics-attempt-2.log；首处σ 1 ULP，诊断全输入13处；不得报告移植零违例 |
| Rust THETA-2 / AC2-25性质部分 | passed | dynamics-attempt-2.log：固定ρ与自适应各7场景；原0.03等阈值未改；完整谱系验收not_run |
| AC2-14，AC2-05 θ | passed（纯C） | 外源非恒等、各自致来源/无事件整体恒等；128前缀完整状态重放 |
| AC2-09 θ | passed（结构/纯C边界） | 签名无x/m/R，拒绝非m=0解释；全管线改变x/m的集成实验not_run |
| I13 | 暂存单元passed；事务集成not_run | 现有ThetaChange before/after/事件ID；L由外层事务写，本卡未接通 |
| 连续检测、h下限、σ时序、τ、数值故障、跳变/慢漂 | passed（固定合成夹具） | dynamics-attempt-2.log；不是误报率或全反馈范围校准 |
| cargo fmt --all -- --check | passed | cargo-fmt-final.log |
| cargo clippy --workspace --all-targets -- -D warnings | passed（修复等价if结构后） | cargo-clippy-final.log；首次lint失败原样留存 |
| cargo test --workspace --locked --no-fail-fast | **failed且未完整收尾** | cargo-workspace.log：kernel26通过/1失败；随后session消失，无最终退出码 |
| Ubuntu完整CI / TS/Python四项目/.NET | not_run（本地未全跑） | 本卡改Rust；不冒称Ubuntu作业全部通过；发布后CI状态以PR checks为准 |
| Windows/GPU/模型/用户谱系 | not_run | 本次范围不含；未操作这些环境 |

命令执行细节、失败和缺失区分见 `evidence/tests/execution-status.json`；按验收编号的result.json只引用原始日志，不重新生成通过结果。

## 规格对照

见 [P5-25 对照表](../../docs/traceability/P5-25.md)，逐项列出I01–I29适用边界、类型检查、运行时拒绝、测试和未集成部分。新增差异登记4项，编号全部“待定”。不更改原规格、任务卡或PR #73。

## 新增依赖

没有新增运行时或测试依赖；复用既有 `libm = 0.2.16`。fixture/诊断仅用Python标准库；未改锁文件。没有复制外部实现或下载模型。

## 待用户确认

- [ ] **阻塞：** THETA-1数值oracle选择，见BLOCKED.md三项选项；原失败必须保留。
- [ ] **阻塞：** 连续高斯均值检测器及其参数/误报检出校准；仅均值移位，未证明任意分布变化可检出。
- [ ] **阻塞：** 同号时长右删失、先验质量、时间精度边界，以及熔断整候选hold和慢漂参数解释。
- [ ] **阻塞：** 临时资源的正式契约与外层事务集成；需高难度卡交叉审查；全工作区/CI终态尚需核验。

[echo-core-work技能](../../.agents/skills/echo-core-work/SKILL.md)明确规定：“差异登记新增的条目。只要有新增，就是阻塞项。”因此这些候选未获批准，不能以合成测试通过替代用户确认或发布新规律。

## 后续建议（不在本任务范围内）

按用户选择处理数值oracle后，再完成THETA-1和完整CI；由后继契约卡/运行时卡接入版本化估计器资源、L原子提交、P5-26暂停复核。P5-30再扩展重尾/相关噪声、仅方差改变、慢变和k_max校准。本轮按用户要求在PR写“请审查”后停止。
