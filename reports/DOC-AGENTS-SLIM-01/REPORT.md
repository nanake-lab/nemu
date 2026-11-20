轻档：只改 `AGENTS.md`，只在 `docs/agents/`、`reports/DOC-AGENTS-SLIM-01/` 下新增文件。

# DOC-AGENTS-SLIM-01 AGENTS.md 瘦身 — 报告

做了什么：把 AGENTS.md 里只在特定场合才用的内容（报告模板、main 增量核对、设计卡与研究卡写法、CI 范围）逐字搬到 `docs/agents/` 下 4 个文件，原位留一行指引；已办完的历史条目搬进 `archive.md`；目录表合并了没有特别规则的行；删掉第 10 节与第 2 节重复的「职责」一条。
对她或对项目意味着什么：规则一条没变，编号一个没动；每个执行者开工要读的 AGENTS.md 从 357 行、39,486 字节降到 291 行、33,703 字节（约少 15%）。
需要你做什么：审查后合并（needs-user）；另看下面「偏离卡面的四处」是否认可。

## 当前状态

完成。基线 origin/main `fc5f4c2`（与卡面基线一致，AGENTS.md 在基线之后没有改动）；搬迁表里的标题全部对得上。

## 改了什么

| 文件 | 内容 |
| --- | --- |
| `AGENTS.md` | 第 3 行末尾加一句；第 2 节目录表合并；第 2、4、6、11 节的 (b) 类条目移走；第 4 节代码块 N3→E4；第 6 节三处、第 9 节一处换成指引；第 10 节删「职责」一条 |
| `docs/agents/report-template.md` | 第 6 节「报告模板：」及代码块 |
| `docs/agents/merge-main-check.md` | 第 6 节「main 增量影响核对」及 1–5 条 |
| `docs/agents/design-research-cards.md` | 第 6 节「设计卡与研究卡怎么写、怎么做」及 4 条 |
| `docs/agents/ci-scope.md` | 第 9 节「CI 的范围」及全部条目 |
| `docs/agents/archive.md` | (b) 类 4 项 |
| `reports/DOC-AGENTS-SLIM-01/agents-refs.md` | 检查 5 的逐行明细 |

## 偏离卡面的四处

都在检查 3 里逐行核过，意思不变：

1. **archive.md 里两个链接改了路径。** 第 4 节「状态更新」一段搬到 `docs/agents/` 后，原来的 `tasks/README.md`、`reports/DOC-ARCHIVE-INVENTORY/REPORT.md` 会指向不存在的位置。卡面的「逐字照搬」和检查 2「链接必须指向存在的文件」在这里冲突。我只改了括号里的路径（加 `../../`），可见文字一字未动。
2. **两行是拆开的，不是整行搬走。** 「真实体验也是验收」的标题和依据句原本在同一行，第 11 节开头句和「试点中遇到过：」也在同一行。卡要求前半留下、后半搬走，所以这两行在 diff 里显示为删除，但「留下的前半 + 搬走的后半」拼起来与原行完全一致。
3. **第 3 行是卡要求的末尾追加。** 原行原样是新行的开头。
4. **目录表保留成表，只合并行。** 卡允许改成列表，但第 2 节「分工」里「下表中原来建议 Claude 的目录」「按下表」和表前一句「执行者一栏」都指着这张表；改成列表就得改这几句的字。合并行能省下一样多的篇幅，这几句也不用动。Antigravity 的 4 行也合成了一行（同样没有特别规则）。

另外，按硬约束 2，新文件只加了开头一行说明，没有加小标题。所以 `archive.md` 里 4 段之间没有标注来源节；要加的话，告诉我一声。

## 检查

| # | 检查 | 结果 |
| --- | --- | --- |
| 1 | `git diff --check origin/main` | 通过 |
| 2 | 新增和改动的相对链接 | 7 个，全部指向存在的文件：AGENTS.md 里 4 个新指引 + 原有的 ADR 0043；archive.md 的 2 个（见上面第 1 处） |
| 3 | 删除行逐行核对 | 见下 |
| 4 | 标题与黄金规则编号 | 第 1–11 节及 3 个三级标题共 15 个，与原文逐字一致；黄金规则 1–17 条逐字一致（对照见下） |
| 5 | 仓库内 `AGENTS` 引用（`reports/` 除外） | 210 行，没有一处失效，明细见 [agents-refs.md](agents-refs.md) |
| 6 | 行数与字节数 | 改前 357 行 / 39,486 字节；改后 291 行 / 33,703 字节 |

### 检查 3：删除行逐行核对

`git diff -U0 origin/main -- AGENTS.md` 的 `-` 行，空行除外，共 **63** 行。

- **在 `docs/agents/` 里找到原文：43 行。** 按文件：report-template.md 21、merge-main-check.md 6、ci-scope.md 6、design-research-cards.md 5、archive.md 5。
- **没找到原文：20 行。** 各自的归属：

| 归属 | 行数 | 说明 |
| --- | --- | --- |
| 目录表改写 | 14 | 见下面目录表对照表 |
| (a) 类重复 | 1 | 第 10 节「职责」，见下面 (a) 类删除清单 |
| N3→E4 改字 | 1 | 第 4 节代码块 `echoctl` 一行 |
| 拆行 | 2 | 「真实体验也是验收」一行：标题留在第 6 节，依据句逐字在 archive.md；第 11 节开头一行：开头句留在第 11 节，「试点中遇到过：」逐字在 archive.md |
| 只改链接路径 | 1 | 第 4 节「状态更新」：把两处链接路径换回原样后，与 archive.md 里那一行逐字一致 |
| 第 3 行末尾追加 | 1 | 原行原样是新第 3 行的开头 |

后三类不在卡面列的三种情况里，原因见「偏离卡面的四处」。没有规则丢失。

### 检查 4：标题与黄金规则对照

标题（代码块外）改前改后逐字相同：

`# AGENTS.md — Nemu 3D 桌面伙伴`；`## 1. 项目一句话`；`## 2. 认领、目录与建议执行者`；`## 3. 黄金规则`；`## 4. 常用命令`；`## 5. 代码规范`；`## 6. 任务完成标准与报告`；`### 证据放哪里、记什么（2026-11-10 用户决定）`；`### 验证与报告按风险分级（2026-10-30 用户决定）`；`### 审查与合并`；`## 7. ADR（架构决策记录）`；`## 8. 技能文件`；`## 9. 子代理与资源调度`；`## 10. Claude（2026-10-19 用户追加，同日按新的第 2、6 节整理；2026-11-12 按现行分工修订）`；`## 11. 云端执行（2026-10-25，按 P5-25 云端试点整理）`。

黄金规则 1–17 条整行改前改后逐字相同：1 一次只做一张任务卡；2 契约是唯一真相源；3 Echo Core 是唯一权威；4 不准制造隐藏状态；5 不准放宽验收；6 不准补写规格；7 美术不靠猜；8 依赖要登记；9 资产要登记；10 Windows 优先；11 不碰密钥；12 Blender 只走无头脚本；13 卡住就停下来问；14 她的数据不是测试数据；15 不替用户操作；16 不改用户级、系统级设置；17 新增之前先做减法。

保留下来的粗体小节名（如「合并前核对四样」「真实体验也是验收（2026-11-04 用户决定）：」）都没改。被换成指引的三处，指引开头的粗体名照卡面写法（去掉了日期）。

### 检查 5：AGENTS 引用

210 行命中：162 行引用的规则仍在原节；35 行泛指 AGENTS.md，不涉及搬走的内容；11 行引用搬走的内容，原节都留有指向新文件的指引；2 行不是规则引用（生成的类型名 `AGENTS_MD`、`.prettierignore`）。没有失效的引用。

引用搬走内容的 11 行：

- **报告模板（第 6 节指引）：** `.github/pull_request_template.md:6`、`tasks/README.md:57`、`tasks/P0-foundation.md:84`、`README.md:105`
- **main 增量核对（第 6 节指引）：** `.github/pull_request_template.md:16`、`tasks/README.md:22`
- **设计卡与研究卡写法（第 6 节指引）：** `tasks/P5-echo-core.md:2401`、`docs/journal/README.md:30`、`docs/journal/README.md:62`、`tasks/requests/DESIGN-LAW-01.md:24`、`tasks/requests/DESIGN-VRAM-01.md:7`

## (a) 类删除清单

第 10 节「**职责（2026-11-12 用户决定）：**」一条，逐句比对：

| 第 10 节原句 | 第 2 节已有 | 处理 |
| --- | --- | --- |
| 和用户一起规划，写派卡提示，审查（第 2 节「分工」）。 | 「分工」Claude：「和用户一起规划，写派卡提示，审查；」 | 删 |
| 不认领实现卡；实现交给 Codex。 | 「分工」Claude：「不认领实现卡。」；Codex：「实现。」 | 删 |
| `tasks/cards.json` 里未完成的卡不再建议 Claude，已完成的卡保留当时的建议执行者。 | 「旧写法的读法」：「未完成的卡……一律按本段读作 Codex；以 `tasks/cards.json` 为准。已完成的卡保留当时的建议执行者。」 | 删：后半逐字相同；前半表述不同，效果相同（未完成的卡都不归 Claude） |
| 文档类工作（card-edit、学习文档等）可以由 Claude 会话直接完成， | 「分工」Claude：「文档类工作（卡片与文档维护 card-edit、学习文档等）可以由 Claude 会话直接完成。」 | 删 |
| 照常开草稿 PR。 | 第 2 节「认领」：「开工时先在 `task/<任务号>-<短名>` 分支上开一个草稿 PR（draft）」，没有区分执行者；第 10 节开头：「Claude……遵守本文件全部规则」 | 删 |

第 10 节其余四条（审查、环境、工具链、Windows CI）没动。

## 目录表对照表

原表 20 行 → 新表 8 行。路径后括号里的说明全部保留。

| 原表行 | 新表行 |
| --- | --- |
| `core/**`（…） \| Codex（`core/contracts/` 只在标注「契约变更」…） | 第 1 行，原样 |
| `core/crates/core-policy/**`、`core-kernel/**`、`experiment-world/**` \| Codex | 第 2 行（Codex 合并行） |
| `workers/**`（irl、drive、model-adapter） \| Codex | 第 2 行 |
| `executors/**`（codex-bridge、office-connector） \| Codex | 第 2 行 |
| `services/**`（voice-sidecar） \| Codex | 第 2 行 |
| `apps/desktop/**` \| Codex | 第 2 行 |
| `packages/terminal/**`（终端客户端） \| Codex | 第 2 行 |
| `packages/desktop-sense/**` \| Codex | 第 2 行 |
| `packages/voice/**`（含 `packages/voice/web/**` 麦克风采集模块） \| Codex | 第 2 行 |
| `tools/**` \| Codex | 第 2 行 |
| `profiles/**`（Echo Core 结构夹具与部署配置） \| Codex | 第 2 行 |
| `apps/stage/**` \| Antigravity | 第 3 行（Antigravity 合并行） |
| `apps/panel/**` \| Antigravity | 第 3 行 |
| `packages/avatar-runtime/**` \| Antigravity | 第 3 行 |
| `packages/body/**`（物理状态机、反射、读出应用） \| Antigravity | 第 3 行 |
| `packages/contracts/**` \| Codex（只在标注「契约变更」…） | 第 4 行，原样 |
| `config/**` \| Codex（`opportunities.json`、`helpers.json` 的修改需要 ADR） | 第 5 行，原样 |
| `spikes/**` \| 由任务卡指定 | 第 6 行，原样 |
| `content/**` \| 用户。…（整行） | 第 7 行，原样 |
| `docs/**`、`tasks/**` \| 文档类工作可由 Claude 会话完成；…（整行） | 第 8 行，原样 |

## 后续建议（不在本任务范围内）

- `README.md:105` 写「16 条黄金规则」，现在是 17 条。
- 瘦身只少了约 15%。AGENTS.md 里最长的部分（第 6 节的分级、审查与合并规则、第 2 节）每张卡都用得上，按本卡「只搬家」的范围不能再搬。要再明显变短，就得改写或合并规则措辞，那需要另开一张卡由你决定。
