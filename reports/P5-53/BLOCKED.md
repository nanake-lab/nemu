# P5-53 第 1/4 部分

重档。排练首次发布涉及权威提交头、读者可见性和恢复。

本轮在原分支实现了排练初始根。开工复核时 GPU 安静锁不存在，本机重负载计数为 0。启动最终工作区 Clippy 前，检查到 1 个其他重负载进程，命令在调用 Cargo 前退出。按派卡要求停止，保留草稿。

| 检查 | 实际结果 |
| --- | --- |
| 开发中版本，真实临时存储 | 公开 runtime 入口联测通过 1 项，10.34 秒。覆盖发布前不可见、完整根读回、重开、幂等与冲突。 |
| 开发中版本，独立写进程退出 | 通过 1 项，14.25 秒。分别在 BeforePublish、SqlCommitted、AfterPublish 终止写进程，恢复后检查待处理 D、消费位置、单一提交与回执。 |
| 当前修订 | `cargo fmt --all -- --check` 与 `git diff --check` 通过。新增的 x/R⁺ 维度核对、准备前完整性检查、初始 Q 降级回读、误配输入与旧备份反例均须在此修订复测。工作区 Clippy 和完整 Rust 回归为 `not_run`。 |

接续条件为本机其他重任务结束。接续时重新检查 GPU 安静锁、实际重负载、PR head 和工作区，再运行两个包的新增测试、工作区 Clippy 和 Rust 回归。验证完成后处理本文件并请求交叉审查。第 2 至第 4 部分仍见草稿 PR 拆分计划。

[开发中版本的原始日志归档](https://github.com/nanake-lab/nemu-evidence/blob/4500602f9ac8c3e85ecbfe290d817e7f0abcda38/P5-53/part-1/preliminary-ef6d71318fcf.zip)，SHA-256 `ef6d71318fcfccbea5f872b707e181f148132fd5bfad97d629b4a4a86cfbe679`。
