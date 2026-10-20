# P0-07 恢复与补证准备（2026-10-20）

## 已核验

- 从 `D:/Dev/nemu` 执行 `Test-Path D:/Dev/nemu-p007` 得到 True；worktree 已登记，分支 `task/P0-07-carrier-probe`，初始 HEAD `2ba7d16c1978cb0f60514cababa617124e077fd6`，初始 status 干净。未 prune、未重新添加。
- fetch 后将 origin/main `5a0f106` 合并为 `62d38d3`（两个父提交 `2ba7d16`、`5a0f106`），已正常推送；没有 rebase 或强推。草稿 PR #27 保持 OPEN / draft。
- 合并提交 CI run 39383584025：Ubuntu 2m43s、Windows 2m5s 均 pass。
- 五个固定修订、15 个 safetensors 的实际大小、SHA-256 均匹配缓存 metadata LFS 和 preparation 清单。逐文件证据 `weights-audit.json`。由 routine_reporter 角色执行机械核验，主代理再解析全部 revision/weight 匹配字段。仅核验本地清单，未重新向远端查询元数据。
- 权重实际位置 `D:/Dev/_leftover/2026-10-20/workers/model-adapter/probe/.cache/models`；本工作树 probe/.cache 通过 NTFS junction 指向旧 cache，git check-ignore 确认忽略。未移动或提交权重，未下载。
- 使用旧隔离环境的 Python 3.12.14；torch 2.10.0+cu128、transformers 4.57.3、bitsandbytes 0.50.2、qwen-tts 0.1.1，与现有锁定版本一致。没有重建或迁移 venv。
- CPU pytest：4 passed in 15.31s（本轮工具执行输出）；Ruff 全通过、mypy --strict 九个文件通过，日志见本目录。没有新增依赖或修改实现。

## 继续路线 1：测量边界

1. 跨内核：扩展到独立中/英/日真实提示与多个 decode 位置；相同 token/KV 输入条件比较参考和原生 RMS，保存最大差、分布及 argmax 变化。零调制同内核对照与跨内核对照分开，不自行冻结生产容差；Instruct 参考 RMS 保留。
2. 显存：先验证 Windows WDDM 的进程计数器是否能提供完整进程归因，登记载体/TTS、Chromium GPU 等进程及时间，记录采样间隔、缺失和失败。allocator 峰值、全卡样本与进程归因分别报告。有限频率采样不能声称捕获任意瞬时峰值；观测能力不足时保留阻塞，不伪造连续精确峰值。
3. 信息性方向观察：保留现有庆祝场景原始数据，补独立中性场景；此前要求的 -4/-2/-1/0/1/2/4 剂量和每档三 seed 必须显式登记为独立探索预算，不改变旧 beta=1 路径或生产配置，不截断/偷偷重标系数。保存全部文本、token、方向与停止原因，并产出本地 HTML 比较页；不能将 token 变化推断成稳定语义方向。正式 v2.0 方向提取与验证留给 P5-31。

以上是待执行补证安排，不是已完成实现或测量。固定 600 ms / 9 GB；不自行选型。首个安静 GPU 窗口估计 45–60 分钟，诊断可能延长；开始前等用户确认停 runner，并检查 Listener/Worker 均退出。窗口内重负载串行，结束后通知恢复 runner。当前没有开始 GPU 窗口。
