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

预算脚本默认每月 60 次 × Ubuntu 20 分钟 = 1200 分钟（按 2000 分钟示例额度为 60%）。2026-10-24 起单次 Ubuntu 检查已接近 20 分钟，作业超时上限改为 35 分钟。
实际额度以账号账单为准；仓库变量 CI_FREE_MINUTES、CI_MONTHLY_RUNS、CI_HOSTED_MINUTES_PER_RUN 可调整。
超过 80% 时输出 GitHub workflow warning。它是预测，不声称获取了真实账单用量。
Windows 自托管时间不计入上述托管分钟估算。

官方参考：[electron-vite](https://electron-vite.org/guide/dev)、
[Electron 安全说明](https://www.electronjs.org/docs/latest/tutorial/security)、
[GitHub runner 安装](https://docs.github.com/en/actions/how-tos/manage-runners/self-hosted-runners/add-runners)。

## 首次开发环境搭建（P0-04）

### 分项运行 doctor

请在仓库根目录的 PowerShell 7 中一次检查一项，并按实际输出处理：

```powershell
pwsh -File tools/doctor.ps1 -Check Git
pwsh -File tools/doctor.ps1 -Check Blender
pwsh -File tools/doctor.ps1 -Check Office
pwsh -File tools/doctor.ps1 -Check OfficeCOM
pwsh -File tools/doctor.ps1 -Check NvidiaDriver
pwsh -File tools/doctor.ps1 -Check Torch
pwsh -File tools/doctor.ps1 -Check LowIL
```

每项检查在独立子进程中运行；默认 30 秒超时，Blender、VRM Add-on、PyTorch 和 Low IL 探针使用更长但有上限的时间。超时会终止该项的进程树。`-Check All` 也会逐项启动并分别计时；恢复或排查系统稳定性时应使用单项命令。

### Blender 与 VRM Add-on

项目要求 Blender LTS ≥ 5.1。本机经无头版本探针确认的首个锁定版本为 **5.2.2 LTS**，精确版本号记录在 `tools/asset-pipeline/blender-version.txt`；P2-03 开工后由该卡接管此文件。请安装与 Blender 5.2.2 兼容的 VRM Add-on，并在 Blender Preferences 中启用。doctor 只启动 Blender 后台 Python 查询版本和插件启用状态，不会打开模型或修改用户文件。

### NVIDIA 驱动、CUDA Toolkit 与 PyTorch

NVIDIA 驱动和 `nvidia-smi` 是 GPU 运行检查项。当前代码与 P0-07 的 PyTorch 运行路径没有自定义 CUDA 扩展编译；预编译的 PyTorch CUDA wheel 自带其所需的 CUDA 运行库，因此 **CUDA Toolkit / `nvcc` 是提示项，不要求为了运行 PyTorch 单独安装**。只有项目新增 CUDA 扩展或从源码编译 CUDA 代码时，才安装与该构建匹配的 Toolkit。参考 [PyTorch 安装命令](https://pytorch.org/get-started/previous-versions/) 和 [PyTorch C++/CUDA 扩展要求](https://docs.pytorch.org/docs/stable/cpp_extension.html)。

model-adapter 的独立 uv 环境尚未记录 Torch 依赖。请在仓库根目录手动安装 CUDA 12.8 wheel：

```powershell
uv pip install --python workers/model-adapter/.venv/Scripts/python.exe torch --index-url https://download.pytorch.org/whl/cu128
uv run --project workers/model-adapter --no-sync python -c "import torch; print(torch.__version__, torch.cuda.is_available())"
```

第二条命令应显示已安装版本和 `True`；若不是 `True`，保留完整输出，检查 NVIDIA 驱动与 PyTorch CUDA wheel 的兼容性。安装完成后再单独运行 `pwsh -File tools/doctor.ps1 -Check Torch`。

### Microsoft Office 与 WPS COM 注册

Microsoft Word、Excel、PowerPoint 桌面版均为必需项。doctor 只读取注册表，不会启动 Office 或 WPS。它会逐项检查 64 位 `Word.Application`、`Excel.Application`、`PowerPoint.Application` 的 COM 服务器路径，并把 32 位视图作为诊断信息。如果 64 位 ProgID 指向 WPS，请先在 Windows“已安装的应用”中对 Microsoft 365/Office 执行“快速修复”，再执行“联机修复”；若注册仍被 WPS 占用，再修复或卸载 WPS 后重跑 `-Check OfficeCOM`。每次修复后重新确认三个 ProgID 指向 `Microsoft Office\...\WINWORD.EXE`、`EXCEL.EXE`、`POWERPNT.EXE`。
### Blender 可执行文件路径

当前 Blender 安装位置为 `D:\blender\blender.exe`。资产管线按 `.agents/skills/blender-headless/SKILL.md` 从 `NEMU_BLENDER` 读取 Blender 路径；本机当前没有设置该变量。为后续 P2-03 使用，请在 PowerShell 7 中执行以下命令，将路径保存到当前用户环境，并在新开的 PowerShell 窗口生效：

```powershell
[Environment]::SetEnvironmentVariable('NEMU_BLENDER', (Join-Path 'D:\blender' 'blender.exe'), 'User')
```

## VRoid 导出设置（P2-03 追加，2026-10-21）

在 VRoid Studio 的导出面板选择 **VRM 1.0**，不要选择 VRM 0.0。导出到 `source/nemu-vroid-export.vrm`，保留 `.vroid` 原工程；本卡管线只读该 VRM。不同 VRoid 版本的选项名称可能变化，以下以导出行为为准，不臆造当前界面上的按钮名称。

| 导出项目 | 设置 | 理由与检查 |
| --- | --- | --- |
| VRM 版本 | 1.0 | 必须带 `VRMC_vrm/specVersion=1.0`；现有管线按 VRM 1.0 属性加工 |
| 标题、作者、许可 | 名称 Nemu；作者填写真实创作者；按内容包许可填写使用/修改/再分发限制 | 不得借管线放宽素材许可；meta 和 manifest 需要完整来源 |
| 姿态与朝向 | 保留标准 humanoid 骨架和默认导出坐标，不手动删骨、缩放或旋转骨架 | 必需骨含胸/上胸、颈、肩、脚趾与全部手指；最终以 +Z 朝向、脚底 y=0 检查 |
| 面数削减 | 用导出面数削减选项控制三角面 ≤100,000；目标 ≤70,000 | 以导出后 asset-lint 的三角面数为准，不能用编辑器多边形数替代 |
| 材质合并 | 启用导出材质合并/图集功能（若此版本提供），目标 ≤12 个材质；保留 MToon | 材质数是警告预算；13 个及以上需记录，不用破坏表情绑定强行合并 |
| 贴图尺寸 | 最大边 2048；如导出面板可统一选尺寸，选 2048 或更小 | 管线仍会按 YAML 上限缩小；显存 RGBA8+mipmap ≤256 MB，贴图张数目标 ≤16 |
| 骨骼削减 | 保留 humanoid 必需骨、手指及头发/衣服的弹簧骨；若有删除未使用骨选项，确认不删除契约要求的骨 | 骨数目标 ≤250；耳/尾最终由 P2-05 验收，不能靠删必需骨减数 |
| 表情/BlendShape | 保留所有口型、眨眼和基础表情，不烘焙成静态网格 | 管线需要绑定真实形态键；自定义表情参数缺失时应报错，由 P2-06 决定形态组合 |
| 第一人称/视线 | 保留默认人形骨骼映射，导出骨骼型视线（若可选） | lookAt type=bone 为契约建议；管线不凭经验猜测眼球旋转范围 |

导出设置只是预算准备，不能保证成品验收。先对中期 VRM 跑 `pnpm asset:lint` 查看实测值，再将经选择的参数填入内容包 build.yaml。P2-03 的 schema/带注释样例在 `tools/asset-pipeline/`，示例数值仅用于官方样例集成测试；本卡不修改 `content/` 和 `content-template/`。
