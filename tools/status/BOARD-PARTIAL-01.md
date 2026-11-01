中档：仅修改看板工具与测试；按本次目录限制将报告放在 tools/status/。

# BOARD-PARTIAL-01

## 当前状态

功能与本机回归完成，全目录 strict mypy 已通过。按用户要求转为可审查，等待审查，不合并。

## 修改

- `delivery.py`：读取 main 文件树及固定 blob 的报告；报告「当前状态」写部分交付/完成/实现，或 BLOCKED 仍有未解除内容时，返回后继请求与卡号。已勾选项、明确已解除章节，以及当前状态声明全部解除的历史阻塞不再计入；旧式表格/正文阻塞仍识别。未命名后继时显示报告路径，不猜依赖。
- `build_status.py`：已合并 PR 的完成判定加入 `partial`；复用 `waiting_on`。部分交付不计入完成数、不放行依赖；原有开放 PR、暂停及 unverified 优先级保留。读取错误中止刷新，不冒充缺失报告或完成。
- `board.html`：仅增加状态筛选和排序中的「部分交付」，复用现有「在等」列。
- 两个测试文件：完成/部分交付、依赖与里程碑、解除阻塞、历史内容、固定 main blob 读取及读取失败回归。
- `status_types.py`：仅声明实际消费/输出的 JSON 字段类型；`build_status.py` 和测试辅助函数采用这些类型，JSON/API 返回处用 `cast` 标注已依赖的结构，保留现有分支与解析行为。为避免类型冲突区分响应变量及正则匹配变量；无新增 Any、忽略注释或检查配置修改。

## 本机检查

Windows / Python 3.12；mypy 1.20.2、Ruff 0.15.6，复用 D 盘现有工具。基线 B 与交付前 fetch 的 M 均为 `ee5c2b6068b9a2200ad5fd48ebbc8e337458c0d5`。

| 命令或检查 | 结果 |
| --- | --- |
| `python -m unittest discover -s tools/status -p 'test_*.py' -v` | 15 passed |
| `ruff check tools/status` | passed |
| `mypy --strict tools/status` | passed，6 files，0 errors |
| 提取 board.html 脚本后 `node --check`，对全部 Python 脚本调用 `compile(..., 'exec')` | passed |
| 本轮代码与 `ffbf0666` 的 AST 对照（剥除类型注解、cast、导入，归一局部变量改名） | 两个修改的 Python 文件可执行 AST 一致；15 项测试断言未改 |
| 真实 main P5-24 报告读取 | 识别部分交付；后继来自 BLOCKED 的请求及 P5-06/P5-26/P5-29，未查询 CI |
| `git diff --check` | passed |

此前全目录 mypy 的 23 条诊断为裸 list/dict 类型参数、JSON 返回 Any、PR details 推断为 bool 后写入审查字符串；本轮按用户追加授权修复。补全类型后暴露的字典展开类型及同名变量推断共 9 条中间诊断也已修复。最终指定检查全部通过，无未完成检查或阻塞。`build_status.py` 原已超过 400 行；类型声明独立放入小模块，避免进一步挤入状态计算脚本。

无新增依赖。报告自然语言没有机器 schema，按现有标题、复选框及显式解除声明读取；未明确解除的 BLOCKED 正文保守保留，不能由某个后继 PR 合并自行推断已解除。本轮类型修复不改变这些行为。需要用户安排审查。请审查。
