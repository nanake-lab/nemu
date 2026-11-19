中档，改动 GitHub Actions 的路径筛选和检查步骤。

# CI-SCOPE-DEMO-01 演示工具 CI 分流

## 当前状态

本地验收通过。GitHub 的 Ubuntu 与 Windows 全量验收结果以 [PR #230 当前提交的检查页](https://github.com/nanake-lab/nemu/pull/230/checks) 为准。

## 交付物

- `ci.yml` 将 `tools/cause-demo/` 归入独立的 `causedemo` 检查，用 Python 标准库运行演示单测。全量运行也包含该检查。
- `windows.yml` 同步调整 push、pull request 路径筛选和桌面组正则。

## 验收

PyYAML 6.0.3 完整解析两个工作流，actionlint 1.7.12 校验通过。actionlint 首次提示既有的 `nemu` 自定义标签未登记，载入现有 runner 标签配置后通过。`git diff --check` 通过。Python 3.12.10 执行 `python -m unittest discover -s tools/cause-demo -p 'test_*.py' -v`，22 项全过。

用 PowerShell 7.6.9 执行两个 YAML 中提取的完整 `Decide scope`，以任务卡路径代替 Git diff。表中 Ubuntu 列列出全部为真的输出。

| 文件清单 | Ubuntu 改动前 → 改动后 | Windows desktop 改动前 → 改动后 |
| --- | --- | --- |
| `tools/cause-demo/plan_speak.py` | node → causedemo | true → false |
| `tools/cause-demo/chat.py` + `core/crates/task-layer/examples/cause_demo.rs` | node、rust → causedemo、rust | true → false |
| `tools/status/build_status.py` | node、status → node、status | false → false |

另测两份工作流改动和手动全量运行，Ubuntu 八个 scope 与 Windows 三组均为真。两处 Windows 事件路径表均只追加了演示目录排除项。

## 待用户确认

请审查演示工具单测的触发范围和 Windows 路径排除。Windows runner 曾离线，已恢复。草稿阶段自动 Windows 检查的门禁按既有策略报告失败，另行发起了全量检查。
