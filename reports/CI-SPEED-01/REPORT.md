中档。涉及 GitHub Actions 的调度与编译缓存。

# CI-SPEED-01 缩短 CI 与 Windows 检查耗时

## 改动前基线

以 GitHub Actions API 截取 CI、Windows 各 20 次最近结论为 success 的运行，分别按运行号和最终 attempt 读取 Jobs。统计任务实际执行的成功步骤，排除 skipped，以步骤起止时刻之差的中位数排名。不同事件与范围混采，样本数见表。工作流总时长从首次创建到最终 attempt 的最后一个 job 完成，再次尝试与排队可能计入。Windows 14/20 次运行执行了 W02、W18，17/20 次运行执行了桌面 e2e。

| 排名 | 平台与步骤 | n | 中位数 / 范围（秒） | 主要工作 |
| ---: | --- | ---: | ---: | --- |
| 1 | Windows · W02 NTFS storage checks | 14 | 1228 / 1130–1451 | 测试，附带 clippy 和编译 |
| 2 | Ubuntu · Rust checks | 13 | 843 / 616–1069 | 测试与编译，附带 clippy、格式检查 |
| 3 | Ubuntu · core-store tests (including crash recovery) | 13 | 552 / 448–736 | 测试，附带编译 |
| 4 | Ubuntu · P5-01 IDL and three-language conformance | 10 | 130.5 / 102–167 | 工具安装、编译、三语言验证 |
| 5 | Windows · W18 gateway Windows components | 14 | 123.5 / 106–155 | 测试，附带 clippy 和编译 |
| 6 | Ubuntu · pnpm lint、typecheck、test、build | 11 | 37 / 31–46 | 静态检查、测试、构建 |
| 7 | Windows · visual | 17 | 33 / 31–35 | 测试 |
| 8 | Windows · e2e | 17 | 32 / 29–36 | 测试 |
| 9 | Ubuntu · W19 optional CPU runtime compile checks | 13 | 22 / 18–28 | 编译与 clippy |
| 10 | Ubuntu · actions/checkout | 20 | 19 / 17–29 | 取代码与 Git LFS 输入 |

最新 main 对照的 W02 中，clippy 12.57 秒、测试编译 17.21 秒，65 个库测试执行 1418.40 秒。W18 相应为 1.28 秒、0.49 秒和 102.47 秒。对应 Ubuntu 的 core-store 测试编译为 15.48 秒。Windows 最近 7 次成功手动运行从首次创建到最后一项结束的中位数为 49 分 29 秒，最大 2 小时 1 分 7 秒（含前序 attempt 与队列）。[Windows 日志](https://github.com/nanake-lab/nemu/actions/runs/44884635331) · [Ubuntu 日志](https://github.com/nanake-lab/nemu/actions/runs/44884635358)

## 当前状态

基线已测。托管 Windows 与 CI 调度改动待验证。
