# PR #49 修复核验，2026-10-22

这些是合成输入上的真实命令输出；没有使用用户排练或正式存储。

| 前缀 / 目录 | 含义 |
| --- | --- |
| baseline-regressions | 原实现加回归：4 测试中 3 失败，别名首构造未触发，因为规范排序先比较编码长度 |
| baseline-alias-equal-length | 改用等长且更早的别名 ID，原实现失败；断言没有放宽 |
| postfix | 初版修复完整检查，106 测试通过；Python 原参考核验、工具链版本、AC2-01 独立失败结果 |
| final | 后续边界保护过度拒绝合法的单个别名：8 个原测试失败，保留原始失败日志 |
| final2 | 纠正保护范围后的最终 Rust 检查：fmt / clippy / 107 测试通过。源码摘要见 source-sha256.json |
| ac2-01-acceptance | 实际 Rust 反例 + 未放宽的 AC2-01 判据，result.json 为 failed，外层核验命令 exit 1；内部诊断测试成功只表示反例复现 |

meta.json 记录命令、工作目录、起止时间和实际退出码。测试时 HEAD 为原审查提交，修复在未提交工作树中；source-sha256.json 对应最终代码。final2 的 PR CI 查询仍针对原 head，不能拿它证明修复提交 CI 通过；推送后的 CI 以 PR 当前检查为准。

复现命令在 core/ 下：`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace --locked -- --nocapture`。根目录执行 `python -X utf8 reports/echo-core-v2-draft/check_law.py`；AC2-01 用 `pwsh -NoProfile -File reports/P5-23/verify_ac2_01.ps1 -EvidenceDirectory <不存在的新目录>`，拒绝覆盖既有证据。
