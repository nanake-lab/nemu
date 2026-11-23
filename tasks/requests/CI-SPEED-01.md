# CI-SPEED-01：缩短 CI 时间，把不需要桌面的 Windows 检查搬到 GitHub 托管机

仓库：GitHub `nanake-lab/nemu`。先完整读一遍 `AGENTS.md`。

**投入：** token 预算没有上限，不要吝惜。该读的文件读完，该测的情况测全，证据给全。不要为了省 token 跳步骤，也不要简化验收。

- **线：** 交付线（基础设施，中档）
- **分支：** `task/CI-SPEED-01`，从 `origin/main` 新建
- **目录：** `.github/workflows/`（`ci.yml`、`windows.yml`，必要时可以新建工作流文件）；`tools/scaffold/` 中与 CI 调度直接相关的脚本；`AGENTS.md` 只改第 6 节中 CI 与 Windows 检查的段落；`tasks/requests/CI-SPEED-01.md`（本卡原文）；`reports/CI-SPEED-01/`
- **不碰：** 产品代码、测试的断言和期限、阈值。不批准任何 fork 的工作流运行。
- **预授权：** 上述目录内的实现层决定由你自己做，在 PR 描述里逐条写明；只有涉及「放宽断言、阈值或期限」「跳过已有检查」时，才停下来问。

## 背景

- 用户本机的自托管 runner 跑 Windows 检查很慢：手动全量运行要 52–80 分钟，单是 W02（NTFS 存储）一步就要约 21 分钟。Ubuntu 的 CI 也要 22–35 分钟。
- 自托管 runner 同一时间只能跑一个作业；开 GPU 窗口时还得停掉它，于是 PR 一个接一个排队。
- 执行者常常手动触发**全量** Windows 运行（`gh workflow run windows.yml` 默认什么都跑），还会在草稿阶段反复跑。
- 仓库是公开的，GitHub 托管的 `windows-latest`、`ubuntu-latest` runner 对公开仓库免费，并且可以并行。

## 1. 先测，再改

从最近 20 次成功的 CI 和 Windows 运行里，按步骤统计耗时（可以用 `gh api .../actions/runs/{id}/jobs`），列出最慢的 10 步，标出每一步的时间花在编译、安装还是测试上。结论写进报告，作为改动前的基线。

## 2. 改动（按测量结果取舍；下面每一项都是**候选**，测量说明没用的就不做，在报告里写明原因）

1. **加 Rust 编译缓存。** Ubuntu 和 Windows 都加（例如 `Swatinem/rust-cache`，锁定版本，登记许可证）。缓存键要覆盖 `Cargo.lock`、工具链版本和目标平台。
2. **把 storage（W02）和 gateway（W18）两组搬到 GitHub 托管的 `windows-latest`。** 这两组只用到 NTFS、命名管道和 DPAPI，不需要用户桌面会话。desktop 组（e2e、视觉回归、隔离冒烟）需要真实桌面，留在自托管 runner 上。搬过去以后，各组作为独立作业并行运行。
   - 托管机核数少，跑得慢。先在托管机上把 W02 和 W18 各跑 10 次，记录每次结果和 SYS01 `r2` 的等待余量。10 次全部通过才能切换。只要有一次失败，就保留自托管，把失败日志和原因写进报告。**不准为了在托管机上通过而改期限。**
   - 必需检查（required checks）的名字如果要改，在报告里写清楚新旧名字的对应关系。仓库设置由用户去改，你只写出要改哪几项。
3. **手动运行默认按范围跑。** `workflow_dispatch` 默认按改动文件决定跑哪几组，和 PR 事件一样；只有显式传 `full=true` 时才全量跑。
4. **Ubuntu CI：** 看看能否按改动范围拆成并行作业（例如 Rust 工作区、Python 工具、TypeScript 包分开跑），或者跳过没受影响的部分。跳过的规则必须保守：拿不准的就跑。

## 3. AGENTS 第 6 节加一条规矩

> Windows 检查每个 PR 只在审查通过、准备合并之前完整跑一次。草稿阶段只跑与本次改动相关的组，不触发全量手动运行。需要 GPU 的检查只在自托管 runner 上跑，并按 GPU 窗口规则执行。

## 4. 验收

- 拿两个代表性 PR 对比改动前后的总耗时（从推送到所有必需检查完成）：一个只改核心存储，一个只改文档。
- 托管 Windows 上 W02 和 W18 各 10 次运行的结果表。
- 自托管 runner 上 desktop 组照常通过。
- `actionlint`（如果仓库已经在用）或工作流语法检查通过。

## 5. 交付

开 PR，写「请审查」后停下；卡住就写 `BLOCKED.md`。
