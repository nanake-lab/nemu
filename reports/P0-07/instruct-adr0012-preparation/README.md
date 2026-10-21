# Instruct-2507 / ADR 0012 收尾准备

部分完成，尚未运行本轮 GPU 测量，未选定载体。只围绕 Instruct-2507 固定修订 cdbee75f17c01a7cc42f958dc650907174af0554。PR #38 已合并为 main bd5d9d8，已 merge 进本分支（e861ffd）。已阅读 ADR 0012 第 1、2 条和任务卡的追加，按该口径收尾，600 ms / 9 GB 不变。全部条件满足前 models.lock 保持 candidates-only，PR #27 保持草稿。

## 预注册与实现

剂量计划 workers/model-adapter/probe/instruct-dose-plan-v1.json 已先提交为 03071ae，早于本轮任何 GPU 工作。比例 0、±0.05、±0.1、±0.2、±0.4；三语最后 prefill 位置与 decode 位置分别报告 h 范数中位数，六者中位数作统一参考。a[0] = ratio × reference / norm(U[:,0])，其余七维为零。同样的三个 seed、48 token / 25 秒、temperature .7、fresh KV。系数先落盘，不依样例挑参数。沿用既有 U、提取对照样本和中性提示。

固定配置：NF4/bf16、无 double quantization、eager、原生 RMS、第 17 层（从 0 开始），attention 残差后、post_attention_layernorm 前、rank 8。

instruct_finish.py 串行执行 3 次独立含 WDDM 采样器的并载测量，再做信息性观察与 comparison.html。每次 1 warmup + 20 条二十字请求，p50/p90 单列，全部失败保留。WDDM 汇总测量进程树；专用与共享各自最大观测相加，≤ 8e9 bytes 通过，同时报告 torch allocated/reserved 峰值。保留采样间隙与驱动范围限制，不宣称连续精确峰值。

跨内核差作为配置属性登记：aten::_fused_rms_norm（PyTorch 2.10.0+cu128）对 Transformers 4.57.3 Qwen3RMSNorm，zh/en/ja 最大 logits 差 0.34375 / 0.4375 / 0.546875，argmax 0/27 变化。候选配置另列 bitsandbytes 0.50.2；以后换内核按迁移处理。

## 本机资产复用

Instruct 3 个 safetensors 和既有并载 TTS 2 个 safetensors 的实际大小、SHA-256 与固定修订 LFS metadata、历史 preparation 清单全部一致。通过忽略的 cache/.venv junction 复用，无下载、无新依赖。

weights-raw.json、weights-stderr.log、weights-wrapper-failures.txt 原样保留：首次 PowerShell 包装器参数失败；授权修正后的哈希子进程退出 0、五文件全部一致，但外层包装器最后添加缺失属性失败，退出 1，顶层 exitCode/endedAt 未写入。主代理独立核对 raw 中全部 actual/expected 哈希、大小与两个修订绑定，结果见 weights-review.json。不能把包装器失败写成全部命令成功。

## 验证

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| Ruff probe 全目录 | 通过 | ruff.log |
| 严格 Mypy 5 个新增 Python 文件（含测试） | 通过 | mypy.log |
| CPU pytest probe | 9 passed | pytest.log |
| pnpm lint / typecheck / test，合并 main 后 | 全部通过；443 tests | pnpm-*.log |
| pnpm probe:chat --help | 通过；未加载模型 | chat-help.log |
| doctor NvidiaDriver | 退出 0 | doctor-gpu.log |

CPU 测试覆盖系数/实际方向范数换算、非零试聊资源绑定、CI 在加载模型前拒绝试聊；未把 CPU 测试当作 GPU 通过。

## 手动试聊（不算验收）

仓库根目录手动运行 pnpm probe:chat，默认 m=0。非零示例：

```powershell
pnpm probe:chat --ratio 0.1 --norms reports/P0-07/instruct-adr0012-v1/direction/norms.json
```

须等 norms.json 实测产生后才可用非零比例。正号沿既有平静→兴奋方向，负号反向，不预选偏好参数。相同固定配置，每轮显示首句延迟；48 token / 25 秒内无句界时照实显示。/reset 清空内存上下文，/exit 退出。对话只在 RAM，不保存对话，不打开排练/正式/Core 存储。仅用户手动启动，CI 禁止试聊；实际聊天未运行。

## 下一步

用户已确认本次安静窗口；准备提交后再次检查 Runner.Listener 和 Runner.Worker 均不存在，再启动 GPU，不运行其他测试。结束通知恢复 runner，列出 PR #27 需要 Windows CI 的提交。实测回来后逐条追加建议；有一项不满足就保留 BLOCKED，不自行补跑挑结果。无新增运行时依赖。
