# CI 下载阻塞（代码修复已交复审）

最新状态：7c82ac22 的代码与契约复审已通过，但 CI #410 仍因同一下载 504 failed。已追加测试前显式安装锁定 Electron 的步骤：最多 3 次、间隔 15 秒，持续失败仍退出 1，未更换源、版本或校验方式。控制流测试通过，等待新 head 远端运行，尚未解除本阻塞。用户已启动 Windows runner，将在保持 draft 的条件下手动运行真实 Windows 检查。

实现 head `ae7b2e14b998c3557ec8c0d159796d03e99c5ef9` 的 [CI #409](https://github.com/nanake-lab/nemu/actions/runs/42580654610) 首次和同提交失败作业重跑均 failed：Electron v44.7.0 Linux 二进制下载 HTTP 504，桌面 settings 测试加载失败，后续检查未运行。

原日志：[attempt 1](evidence/review-ci-409-attempt1.log)、[attempt 2](evidence/review-ci-409-attempt2.log)。没有修改 Electron 版本、下载源、测试或阈值。本轮契约回归、三语言 14,755 例与本机语言检查通过；CI #408 通过完整三语言验收后暴露的独立 mypy 搜索路径遗漏也已修复，两条严格 mypy 命令本机通过。

Windows 因保持 draft 跳过实际作业，汇总门禁 failed；不视为通过。PR 不可据本机通过直接合并。

后续选项：

1. 下载恢复后，在同一最终 head 重跑失败作业；代码无需改变，但须等待完整 CI 结果。
2. 若下载持续失败，另开 CI 基础设施排查；需额外范围和时间，不在本卡更换依赖或绕过测试。

P1 修复已获原审查方通过；后续 CI 准备步骤仍待核验。以最终 head 的真实 Ubuntu 与 Windows 检查结果解除本阻塞，保持草稿等待审查。
