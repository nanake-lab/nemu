# CI-SCOPE-DEMO-01：改演示工具时不再触发 Node 和 Windows 桌面检查

- **线：** 交付线
- **执行者：** Codex（GPT-6.2 Sol，中）
- **前置：** DEMO-CAUSE-04 已合并（main 上有 `tools/cause-demo/test_plan_speak.py`）。没有就停下告诉用户。
- **分支：** `task/CI-SCOPE-DEMO-01`，从 `origin/main` 新建
- **目录：** `.github/workflows/ci.yml`、`.github/workflows/windows.yml`；`reports/CI-SCOPE-DEMO-01/`；`tasks/requests/CI-SCOPE-DEMO-01.md`（本卡原文）

## 为什么
`tools/cause-demo/` 是纯 Python 标准库脚本，跟 Node、Electron、桌面应用都无关。但两个工作流都按 `tools/` 前缀把它当成 Node / 桌面改动：
- `ci.yml` 会跑整套 pnpm 安装、lint、typecheck、test、build；
- `windows.yml` 会在用户本机开窗口跑桌面检查。

结果是改一次演示工具，CI 要多跑十几分钟，还要占用户的电脑。

## 改法

### `ci.yml`（Decide scope 步骤）
1. 在 node 的那条判断之前，加一条：路径匹配 `^tools/cause-demo/` 时，设 `$scope.causedemo = $true`，然后 `continue`。这样它不会再进 node 判断。
2. `$scope` 初始化里加 `causedemo = $false`，并按现有写法把它输出成 step output。
3. 在 status 那一步旁边，照它的写法加一步：
   ```yaml
   - name: cause-demo unit tests
     if: steps.scope.outputs.causedemo == 'true'
     run: python -m unittest discover -s tools/cause-demo -p 'test_*.py' -v
   ```
4. 如果 `$full` 为真，`causedemo` 也要跑。按现有其他 scope 处理 full 的方式照做。

### `windows.yml`
1. `push` 和 `pull_request` 两处 `paths` 列表里，都在 `'!tools/status/**'` 旁边加 `'!tools/cause-demo/**'`。
2. Decide scope 里那条 `'^tools/(?!status/|asset-pipeline/|texture-fix/|scaffold/ci-budget\.mjs$)'` 的排除列表加上 `cause-demo/`。
3. 文件开头的注释里，补一句演示工具不进桌面组。

其余任何检查、步骤、触发条件都不改。

## 验收
- 两个 YAML 用解析器完整读一遍，不报错。
- 用 pwsh 把 `ci.yml` 的 Decide scope 逻辑对下面三组文件清单各跑一遍，结果写进报告（可以把该步骤脚本复制到临时文件里，用假的文件清单调用）：
  1. 只有 `tools/cause-demo/plan_speak.py` → 只有 `causedemo` 为真；
  2. `tools/cause-demo/chat.py` 加 `core/crates/task-layer/examples/cause_demo.rs` → `causedemo` 和 `rust` 为真，`node` 为假；
  3. `tools/status/build_status.py` → 结果与改动前完全相同。
- `windows.yml` 的那条正则，对上面三组也各写一行结果：1、2 不进 desktop，3 与改动前相同。
- 本 PR 改了工作流，会触发全量检查，这是预期的。全部通过才算完成。

## 交付
- `reports/CI-SCOPE-DEMO-01/REPORT.md`：一页以内，改了什么、三组清单的结果。
- 开 PR，标题 `ci(CI-SCOPE-DEMO-01): 演示工具不再触发 Node 和桌面检查`，写「请审查」后停下。不合并，不取下一张卡。
