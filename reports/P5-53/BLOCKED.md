# P5-53 第 2/4 部分的合并阻塞

重档，涉及核心提交与存储。实现提交 `53d3d2fb` 的 Windows 临时库和真实桌面联测 3 项、网关回归 49 项、终端测试 7 项通过。两条认证连接中，只有发起会话收到四个连续输出块；取消后保留这四块。本次从输入到第一个合成块实测 1164.4 ms。[私有原始证据](https://github.com/nanake-lab/nemu-evidence/blob/41a08c8864c2ae6fe7b5457f80d438c62f4554df/P5-53/part-2/native-53d3d2fb-27dbc9ce.zip)

Ubuntu 工作区 Clippy 在 `core/crates/semantic-core/src/irl_tests.rs:181` 报 E0659。已比对 main `ecee0b0e` 和分支的编译日志、测试文件及两个类型定义，失败位置相同。[跨卡请求](../../tasks/requests/P5-53-semantic-core-build.md) · [两次原始日志](https://github.com/nanake-lab/nemu-evidence/blob/f607eedf64dc61518f42466f7f1dfcda8dc135ee/P5-53/part-2/ubuntu-rust-e0659-ec929e15.zip)

可由 P5-49 在现有接续 PR 中限定测试导入，一并核验；也可单独派一张编译修复卡，使依赖该测试的 PR 提早恢复检查，代价是多一次独立审查。修复入 main 后按第 6 节同步并重跑。PR 保持草稿，完整 Windows 工作流和首次真实对话验收均为 `not_run`。
