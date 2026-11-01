中档：仅修改看板工具与测试；按本次目录限制将报告放在 tools/status/。

# BOARD-PARTIAL-01

## 当前状态

功能与本机回归完成，等待审查。全目录 strict mypy 仍有 main 已存在的类型问题，不能声称静态检查全部通过；PR 保持草稿，不合并。

## 修改

- `delivery.py`：读取 main 文件树及固定 blob 的报告；报告「当前状态」写部分交付/完成/实现，或 BLOCKED 仍有未解除内容时，返回后继请求与卡号。已勾选项、明确已解除章节，以及当前状态声明全部解除的历史阻塞不再计入；旧式表格/正文阻塞仍识别。未命名后继时显示报告路径，不猜依赖。
- `build_status.py`：已合并 PR 的完成判定加入 `partial`；复用 `waiting_on`。部分交付不计入完成数、不放行依赖；原有开放 PR、暂停及 unverified 优先级保留。读取错误中止刷新，不冒充缺失报告或完成。
- `board.html`：仅增加状态筛选和排序中的「部分交付」，复用现有「在等」列。
- 两个测试文件：完成/部分交付、依赖与里程碑、解除阻塞、历史内容、固定 main blob 读取及读取失败回归。

## 本机检查

Windows / Python 3.12；mypy 1.20.2、Ruff 0.15.6，复用 D 盘现有工具。基线 B 与交付前 fetch 的 M 均为 `ee5c2b6068b9a2200ad5fd48ebbc8e337458c0d5`。

| 命令或检查 | 结果 |
| --- | --- |
| `python -m unittest discover -s tools/status -p 'test_*.py' -v` | 15 passed |
| `ruff check tools/status` | passed |
| `mypy --strict tools/status/delivery.py tools/status/test_delivery.py` | passed，2 files |
| `mypy --strict tools/status` | 未通过：23 条诊断；main 原始三脚本复核为 22 条，测试辅助函数签名换行使原同一行合并的缺类型参数诊断分成两条；问题位置/类别均来自原有代码，未新增忽略或放宽检查 |
| 提取 board.html 脚本后 `node --check` | passed |
| 真实 main P5-24 报告读取 | 识别部分交付；后继来自 BLOCKED 的请求及 P5-06/P5-26/P5-29，未查询 CI |
| `git diff --check` | passed |

全目录 mypy 原有问题为裸 list/dict 类型参数、JSON 返回 Any、PR details 推断为 bool 后写入审查字符串；不在这张状态修复卡中扩展为全目录类型重构。`build_status.py` 原已超过 400 行，本次只在既有状态计算加入交付判定；报告解析另放独立小模块。

无新增依赖。报告自然语言没有机器 schema，按现有标题、复选框及显式解除声明读取；未明确解除的 BLOCKED 正文保守保留，不能由某个后继 PR 合并自行推断已解除。需要用户安排审查，并决定既有类型问题的后续处理。请审查。
