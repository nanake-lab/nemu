# P0-01 开发环境搭建

要求：Git + Git LFS、Node 24、pnpm 12.11.2、Rust 1.99.0（MSVC 工具链及 VS C++ Build Tools）、
Python 3.12、uv 0.12.15、.NET SDK 10.0.401、PowerShell 7。模型和内容仓不是脚手架的运行依赖。

```powershell
git lfs install
corepack enable
corepack prepare pnpm@12.11.2 --activate
pnpm install --frozen-lockfile
pnpm exec playwright install chromium
pnpm lint
pnpm typecheck
pnpm test
pnpm build
pnpm test:e2e
pnpm test:visual
Push-Location core
rustup show
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
Pop-Location
foreach ($project in @('workers/irl','workers/drive','workers/model-adapter','services/voice-sidecar')) {
    Push-Location $project
    uv sync --locked
    uv run ruff check .
    uv run mypy --strict src tests
    uv run pytest
    uv build
    Pop-Location
}
dotnet build executors/office-connector/OfficeConnector.csproj --configuration Release
```

运行应用：`pnpm dev`。只运行舞台：`pnpm dev:stage`（127.0.0.1:5174）。
面板：`pnpm dev:panel`（127.0.0.1:5175）。两窗口是占位，不启动核心、worker 或模型。
父进程为 Electron 创建临时 userData，等 Electron 退出后清理；不使用排练或正式存储。
尚未实现的根命令说明所属任务并返回 2；echoctl 同样拒绝所有业务操作。

## Windows 自托管 runner（P0-05 第二部分）

1. 确认仓库保持私有，只给 nemu 注册 runner，不注册组织级 runner。
2. GitHub 仓库 Settings → Actions → Runners → New self-hosted runner，选择 Windows x64。
3. 在专用目录中下载页面给出的 runner 包，核对 SHA-256，再解压。不要使用正式 Nemu 数据目录。
4. 运行页面提供的 config.cmd，URL 为本仓库，标签加 `nemu`；注册 token 只在本地使用，不保存进脚本、日志或仓库。
5. 用普通用户的交互式桌面会话运行 `run.cmd`。Electron 测试不能用没有桌面的 Session 0 服务代替。
   一次性验证可用 `--ephemeral` 注册；任务结束后 runner 自动注销。
6. 确保该会话 PATH 能找到 pwsh、Node、pnpm、dotnet、Git LFS；首次运行前完成上面的本机检查。
7. CI 的 Windows job 仅运行仓库所有者触发的同仓库代码。不得改为 pull_request_target 后执行外部 PR 代码。
8. 在 Actions 查看 Ubuntu 和 Windows 两组任务。测试只创建临时目录。
   Windows 组测试命名管道字节往返、测试进程降低到 Low IL 后写入被拒绝、Electron 双窗口和软件渲染视觉回归。
   这些环境烟测不等于 P5 的核心权限验收。

## 视觉基准

`pnpm test:visual -- --update-candidates` 只输出候选到 reports/P0-01/visual，不更新基准。
初始占位窗口基准位于 apps/stage/tests/visual/**golden**/，由本卡建立；后续修改已有基准需要用户批准。
比较使用 SwiftShader、固定 420×600 / DPR 1、白黑两背景，像素须完全相同。
它仅验证脚手架，不包含 VRM、动画、DPI 或穿透验收。

## CI 额度

预算脚本默认每月 60 次 × Ubuntu 12 分钟 = 720 分钟（按 2000 分钟示例额度为 36%）。
实际额度以账号账单为准；仓库变量 CI_FREE_MINUTES、CI_MONTHLY_RUNS、CI_HOSTED_MINUTES_PER_RUN 可调整。
超过 80% 时输出 GitHub workflow warning。它是预测，不声称获取了真实账单用量。
Windows 自托管时间不计入上述托管分钟估算。

官方参考：[electron-vite](https://electron-vite.org/guide/dev)、
[Electron 安全说明](https://www.electronjs.org/docs/latest/tutorial/security)、
[GitHub runner 安装](https://docs.github.com/en/actions/how-tos/manage-runners/self-hosted-runners/add-runners)。
