本文件从 AGENTS.md 第 9 节搬出，规则效力与 AGENTS.md 相同。

**CI 的范围（2026-10-20 按用户要求追加）：**

- 只改文档、任务卡、报告或 `content` 子模块指针的 PR 和提交，不跑 CI（`ci.yml` 的 `paths-ignore`）。
- `docs/echo-core/original/` 或 `tasks/cards.json` 有改动时，由 Docs guard（`docs-guard.yml`）检查：前者一律不允许改，后者检查能否解析、依赖和里程碑是否存在。
- Windows 检查在 `windows.yml` 里，在你的电脑上运行，按改动只跑相关的部分：桌面（传输、隔离、端到端、视觉回归）和存储（W02），对应的目录写在该文件开头。纯 Linux 就能验证的改动（如 core-kernel 的数学部分、文档、研究报告）不触发它。草稿 PR 不跑 Windows 检查，转为可审查时自动运行；草稿阶段需要时，用 `gh workflow run windows.yml --ref <分支>` 手动触发，手动触发默认全量。
- 转为可审查以后还要连续修改时，先转回草稿，改完再转回，避免每推一次就在用户电脑上跑一轮。
- 新增一类 Windows 检查时，同时把它涉及的目录加进 `windows.yml` 开头的路径列表和「Decide scope」步骤。
