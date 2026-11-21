中档。涉及 GitHub Actions 的调度与编译缓存。

# CI-SPEED-01 缩短 CI 与 Windows 检查耗时

## 当前状态

实现与对照验收完成，PR 等待审查。Ubuntu 按范围并行，并为两组 Rust 作业启用编译缓存。Windows 默认按完整改动选择组，纯登记和文档改动不占用桌面 runner。混合改动的自托管 desktop 已通过。托管 W18 触发了切换失败条件，W02 和 W18 继续由自托管执行。正式全组 Windows 按第 6 节安排在审查通过后。

## 改动前基线

以 GitHub Actions API 截取 CI、Windows 各 20 次最近结论为 success 的运行，分别按运行号和最终 attempt 读取 Jobs。统计任务实际执行的成功步骤，排除 skipped，以步骤起止时刻之差的中位数排名。不同事件与范围混采，样本数见表。工作流总时长从首次创建到最终 attempt 的最后一个 job 完成，再次尝试与排队可能计入。Windows 14/20 次运行执行了 W02、W18，17/20 次运行执行了桌面 e2e。

| 排名 | 平台与步骤 | n | 中位数 / 范围（秒） | 主要工作 |
| ---: | --- | ---: | ---: | --- |
| 1 | Windows · W02 NTFS storage checks | 14 | 1228 / 1130–1451 | 测试，附带 clippy 和编译 |
| 2 | Ubuntu · Rust checks | 13 | 843 / 616–1069 | 测试与编译，附带 clippy、格式检查 |
| 3 | Ubuntu · core-store tests (including crash recovery) | 13 | 552 / 448–736 | 测试，附带编译 |
| 4 | Ubuntu · P5-01 IDL and three-language conformance | 10 | 130.5 / 102–167 | 工具安装、编译、三语言验证 |
| 5 | Windows · W18 gateway Windows components | 14 | 123.5 / 106–155 | 测试，附带 clippy 和编译 |
| 6 | Ubuntu · pnpm lint、typecheck、test、build | 11 | 37 / 31–46 | 静态检查、测试、构建 |
| 7 | Windows · visual | 17 | 33 / 31–35 | 测试 |
| 8 | Windows · e2e | 17 | 32 / 29–36 | 测试 |
| 9 | Ubuntu · W19 optional CPU runtime compile checks | 13 | 22 / 18–28 | 编译与 clippy |
| 10 | Ubuntu · actions/checkout | 20 | 19 / 17–29 | 取代码与 Git LFS 输入 |

最新 main 对照的 W02 中，clippy 12.57 秒、测试编译 17.21 秒，65 个库测试执行 1418.40 秒。W18 相应为 1.28 秒、0.49 秒和 102.47 秒。对应 Ubuntu 的 core-store 测试编译为 15.48 秒。Windows 最近 7 次成功手动运行从首次创建到最后一项结束的中位数为 49 分 29 秒，最大 2 小时 1 分 7 秒（含前序 attempt 与队列）。[Windows 日志](https://github.com/nanake-lab/nemu/actions/runs/44884635331) · [Ubuntu 日志](https://github.com/nanake-lab/nemu/actions/runs/44884635358)

## 调度变更

- Ubuntu 沿用原有的改动分类和检查命令。Rust 工作区、core-store、Node／Python／.NET 工具分别运行，最后由 `Ubuntu static checks and unit tests` 汇总。Rust 工作区与 core-store 的检查相互独立。两者分别恢复编译缓存，缓存键包含目标系统、架构、工具链、Cargo.lock 和 Cargo 配置。禁用增量编译产物缓存以减小归档。
- Windows 在 GitHub 托管机比较完整 PR、push 或手动触发分支的改动，选中至少一组后才分配自托管 runner。手动触发默认按范围，`full=true` 显式选中全部。缺少比较基准或无法解析文件名时选中全部。
- Windows 纯登记改动免跑 desktop。整次改动须只包含 `manifest.json`、Markdown 或常见许可证文件。混入其他文件后沿用原路径规则。变更重命名按删除和新增分别计入。
- W02、W18 和 desktop 保留自托管。最近一次基线的 W02 编译约占 1451 秒步骤的 2%。W18 已使用本机持久目标目录，因此自托管增设 GitHub Rust 缓存暂不采用。
- 保留 `Ubuntu static checks and unit tests` 和 `Windows desktop and isolation` 两个汇总名称。交付期间查询传统分支保护与分支 rules API，未发现 main 配置了 required status checks。无需改仓库设置。
- 第 6 节规定在需要 Windows 验收的 PR 审查通过后完整跑一次。草稿阶段按改动范围触发。GPU 使用现有窗口规则。

## Ubuntu 缓存实测

同一 CI head 冷跑与复跑均通过。Rust 工作区与 core-store 两个 job 分别命中与本次工具链和 Cargo.lock 匹配的缓存。下表耗时来自 Cargo 的编译报告和 Actions 缓存步骤，单位为秒。[冷跑和复跑](https://github.com/nanake-lab/nemu/actions/runs/44916280895)

| 作业 | 冷跑主要编译 | 复跑缓存恢复 | 复跑主要编译 |
| --- | ---: | ---: | ---: |
| Rust 工作区 | 66.76 | 6 | 48.54 |
| core-store | 23.56 | 3 | 7.94 |

从首个作业开始到汇总通过，冷跑 14 分 01 秒，复跑 17 分 16 秒。缓存收益体现在编译阶段，两次整轮耗时受测试运行时长影响。是否缩短端到端耗时见同一存储改动的 PR 对照。

## Windows 托管试验

试验分别分配 10 台隔离的 `windows-latest` runner 给 W02 和 W18。每台运行原来的 clippy 与 suite，保留运行时限和测试断言。W18 仅将自托管机特有的磁盘路径替换成托管机临时路径。[20 项试验](https://github.com/nanake-lab/nemu/actions/runs/44915308139)

W02 **10/10**。每轮 65 个库测试、3 个文档测试和 clippy 均通过。表中是库测试时间，范围 889.06–1419.17 秒，中位数 1310.67 秒。

W18 **0/10**。每轮 44 项通过、3 项失败，失败均是命名管道或 DPAPI 文件的 DACL 文本比较。实际文本使用 Windows SDDL 别名 `LA`，断言预期以 `-500` 结尾的完整 SID。微软定义了 `LA` 为本机 Administrator 的 SID 别名。根据切换条件保留自托管。参见 [BLOCKED.md](BLOCKED.md) 和 [微软的 SID 字符串说明](https://learn.microsoft.com/en-us/windows/win32/secauthz/sid-strings)。

SYS01 的 `r2` 来自另外 10 台托管机上的同一个测试用例。每台先在临时 checkout 插入单调时钟观测，仅计时原来的 `peer.reply(&r2)` 提交回执读取及检查。原有 10 秒期限和断言照常执行。十个用例均通过，耗时 4.072–6.498 秒，余量 3.502–5.928 秒。试验包装的结果解析误要求标记在行首，libtest 把用例名写在标记前，导致十个作业报失败。表中时长取各自留存的完整输出。正式全包 W18 与补测 SYS01 的行号分别编号，不代表同一台 VM。[SYS01 补测](https://github.com/nanake-lab/nemu/actions/runs/44915879880)

| 序号 | W02 库测试 | W18 用例数 | W18 测试秒数 | 独立 SYS01 等待 / 余量（ms） |
| ---: | --- | --- | ---: | ---: |
| 1 | 通过，1407.54 s | 44 过 / 3 败 | 72.32 | 5624.199 / 4375.801 |
| 2 | 通过，1392.64 s | 44 过 / 3 败 | 93.15 | 6497.591 / 3502.409 |
| 3 | 通过，1129.42 s | 44 过 / 3 败 | 84.18 | 4716.852 / 5283.148 |
| 4 | 通过，1384.40 s | 44 过 / 3 败 | 103.10 | 5300.264 / 4699.736 |
| 5 | 通过，1099.44 s | 44 过 / 3 败 | 104.91 | 5058.058 / 4941.942 |
| 6 | 通过，1236.93 s | 44 过 / 3 败 | 106.85 | 4072.451 / 5927.549 |
| 7 | 通过，1412.92 s | 44 过 / 3 败 | 102.12 | 5694.095 / 4305.905 |
| 8 | 通过，1419.17 s | 44 过 / 3 败 | 106.63 | 5044.869 / 4955.131 |
| 9 | 通过，1138.21 s | 44 过 / 3 败 | 74.44 | 4876.614 / 5123.386 |
| 10 | 通过，889.06 s | 44 过 / 3 败 | 84.01 | 6350.215 / 3649.785 |

## 代表性 PR 对照

两个临时 PR 只添加无运行时用途的观测文件。旧版和新版保留同一有效文件差异，分别对照旧 main 和候选工作流。计时从开始推送到本次关联的 CI 与 Windows 最后一项结束。初次推送的起点来自紧邻 `git push` 前写下的记录时间，包含临时 PR 的打开时间。仓库当时未配置 required status checks，因此取两项既有汇总检查。

| 样本 | 改动前 | 改动后 |
| --- | --- | --- |
| 只改 core-store 路径的 [#256](https://github.com/nanake-lab/nemu/pull/256) | 总计 **25 分 58 秒**。Ubuntu 18 分 19 秒，Windows 25 分 49 秒 | 总计 **27 分 29 秒**。Ubuntu 17 分 34 秒，Windows 27 分 14 秒 |
| 只改 apps 内 Markdown 的 [#257](https://github.com/nanake-lab/nemu/pull/257) | 无对应 check run 或 workflow run，提交 `1992797e` | 无对应 check run 或 workflow run，提交 `4b1e4e5f` |

存储样本中的 Ubuntu 缩短 45 秒，整体增加约 90 秒。Windows 两轮均通过 W02／W18，desktop 组均未被选中。新版中 W02 从 1317 秒降到 1290 秒，W18 从 123 秒降到 99 秒。此次托管范围作业用时 60 秒，自托管的 `Set up job` 用时从 13 秒增至 100 秒。两项均计入本轮实测，耗时上升集中在准备阶段。[旧 Ubuntu](https://github.com/nanake-lab/nemu/actions/runs/44916470920) · [新 Ubuntu](https://github.com/nanake-lab/nemu/actions/runs/44918934118) · [旧 Windows](https://github.com/nanake-lab/nemu/actions/runs/44916470910) · [新 Windows](https://github.com/nanake-lab/nemu/actions/runs/44918934098)

额外的 `manifest.json` 与 Markdown 组合完成 Windows 范围判定与汇总，物理 runner 跳过。随后加一个普通文件，Windows scope 选中 desktop。同步 main 后，以 #254 的真实文件差异验证 `LICENSE`、`NOTICE` 和多个 manifests 同时变化时仍免跑 desktop。[纯登记运行](https://github.com/nanake-lab/nemu/actions/runs/44916723257) · [仓库许可登记](https://github.com/nanake-lab/nemu/pull/254)

## 验收

| 项目 | 结果 |
| --- | --- |
| 工作流语法与表达式 | `actionlint` 1.7.12 通过 |
| Windows 范围 | `pwsh -NoProfile -File tools/scaffold/windows-ci-scope.tests.ps1` 12/12，通过纯登记、混合、全分支 diff、缺失基准等情况 |
| 检查门槛 | 对 Ubuntu/Windows 汇总脚本执行 20 项结果及来源组合，均符合预期。原有执行命令逐项保留 |
| 纯登记 | Windows 托管范围和原汇总通过，物理 runner 未分配。追加普通文件后运行并通过 desktop。[纯登记运行](https://github.com/nanake-lab/nemu/actions/runs/44916723257) |
| Ubuntu | 冷跑与缓存复跑均通过，存储单路径对照的 Rust 工作区、core-store 和汇总均通过。[CI 结果](https://github.com/nanake-lab/nemu/actions/runs/44916280895) |
| Windows 自托管 desktop | 传输、隔离、e2e、视觉回归均通过，随运行上传来源校验证据。[desktop 运行](https://github.com/nanake-lab/nemu/actions/runs/44916947919) |
| 同步 main 后的验收 | 以 `996a4e47` 同步。新增 `NOTICE` 识别，12 项范围回归和 #254 的实际差异验证通过。Ubuntu 汇总查看 [PR 当前检查](https://github.com/nanake-lab/nemu/pull/255/checks) |

## CI 依赖

`actions/cache` v4.3.0，MIT，固定为 `0057852bfaa89a56745cba8c7296529d2fc39830`。[版本源码与许可证](https://github.com/actions/cache/tree/v4.3.0)

## 并行中的 PR

对 #242 的已知 head `a77e3e64` 做只读试合并，`windows.yml` 的 scope 代码有冲突。两个 PR 合入 main 前需核对合并顺序。第二个合并的 PR 需保留 P5-13 候选的 carrier 路径、独立选择和 native 步骤，并在整合后的 Windows 工作流重跑受影响组。[P5-13](https://github.com/nanake-lab/nemu/pull/242)

## 证据

原始基线、托管试验日志、桌面记录、计时脚本与两组 PR 对照已归档。见 [文件和 SHA-256 索引](EVIDENCE.md)。
