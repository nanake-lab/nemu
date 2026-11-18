**档位：重档（A1 试点包含 GPU 训练与行为测量）**

# A1 接触时机：第一轮结果

训练前的基座在 H0 的 128 组相反历史中，两边始终选同一种行动。训练后，常见规律正确率 **79.6875%**，相反规律 **67.1875%**，H0 翻转率 **41.40625%**。相反规律组高于只背训练集多数规则时在反转测试集上的 25%；常见组仍高于相反组。核心问题尚在于：模型可能只认历史里的关键句，未必真的把回应与当前时间对应起来。

## 结果

| 指标 | 训练前 | 训练后 |
|---|---:|---:|
| 常见组二选一正确率（K=0） | 50% | **79.6875%（51/64）** |
| 相反组二选一正确率（K=1） | 50% | **67.1875%（129/192）** |
| 测试集四选一正确率 | 50% | **65.625%（168/256）** |
| H0 两边都选对的组占比（翻转率） | 0% | **41.40625%（53/128）** |
| H0 两边选同一语义行动的组占比 | 100% | **58.59375%（75/128）** |

训练前、训练后的终端原始指标：

```text
BASELINE metrics: {"four": 0.5, "h0_flip": 0.0, "h0_same_action": 1.0, "two_common": 0.5, "two_reverse": 0.5}
POST metrics: {"four": 0.65625, "h0_flip": 0.4140625, "h0_same_action": 0.5859375, "two_common": 0.796875, "two_reverse": 0.671875}
post test: per-item logprobs D:\Dev\cache\R-BRAIN-01\pilot-a1\post-test.jsonl sha256=e894efb50c3472d5d6b0e035db64cd163fe8ccfed860c8aaa38a09b4e5554c08
post h0: per-item logprobs D:\Dev\cache\R-BRAIN-01\pilot-a1\post-h0.jsonl sha256=1e402b186b3757bf2105462c3de1deae96868004073967f0005bd8c11dfe6c16
POST: PASS; summary_sha256=405ada95bcbab3e5898b4c558bf8437ff16cd4c0a5bd69e088e3cb548e9ce10d
```

## 训练事实

- **1024 块训练，1 epoch，64 次优化器更新**；训练耗时 **1414.1 s**；CUDA reserved 峰值 **4032.0 MiB**。
- **LoRA B**：初始范数为 0；训练后最小 **0.0015642308**，最大 **0.049039237**。
- **LoRA 覆盖**：合计 **248 个模块**，线性注意力 **120**、MLP **96**、完整注意力 **32**。从 adapter safetensors 键名汇总出的 **12 种模块名后缀**如下（这是已提供的去重后缀清单，不冒充 248 条带层号的完整路径）：

  ```text
  modules 248
  {'linear-attn': 120, 'mlp': 96, 'full-attn': 32}
  ['linear_attn.in_proj_a', 'linear_attn.in_proj_b', 'linear_attn.in_proj_qkv', 'linear_attn.in_proj_z', 'linear_attn.out_proj', 'mlp.down_proj', 'mlp.gate_proj', 'mlp.up_proj', 'self_attn.k_proj', 'self_attn.o_proj', 'self_attn.q_proj', 'self_attn.v_proj']
  ```
- **基座 safetensors**：训练前后 SHA-256 一致。两片原始权重对应 P0-12 清单：
  - `model.safetensors-00001-of-00002.safetensors`：`26a93f066e1916adb13453dae5a0c707c0fbc71299ed98779571a907b8e74c61`
  - `model.safetensors-00002-of-00002.safetensors`：`cb544bd9bfae93dc59b0f22b292f5933573854a7f9b97835c67060d7d910e188`
- **参数**：Qwen3.5-4B NF4；LoRA all-linear、r=8、alpha=16、dropout=0；DPO beta=0.1、lr=5e-6、microbatch 1、累计 16、最大长度 512、训练种子 61001。脚本：[ `pilot_a1.py`](pilot_a1.py)。不涉及核心四乎评价；偏好是试点脚本的结构标签。

## 冻结数据与逐题证据

| 文件（本地 `D:\Dev\cache\R-BRAIN-01\pilot-a1\`） | SHA-256 |
|---|---|
| `train.jsonl` | `01e09ca17abf62ff0f442aed03d5af73f440d13706e8fe3c53fa45ee05b58821` |
| `test.jsonl` | `e7a9d6ad2ad0c420ec5271926372a7b8922dc80be3c8aec103e1885c88d1ee63` |
| `h0.jsonl` | `b2c2abdf120f318b88736278696dafeca6aed8d2329658609eb80d9df3b128e4` |
| `baseline-test.jsonl` | `9dace52c87301ca3d37cb2266d8340f867b3b168756cb938d23da73fec6289d0` |
| `baseline-h0.jsonl` | `3fb9b8e4c5269e14be638a78c893d7681896a5779d0182c17e0b8ee41327aa23` |
| `post-test.jsonl` | `e894efb50c3472d5d6b0e035db64cd163fe8ccfed860c8aaa38a09b4e5554c08` |
| `post-h0.jsonl` | `1e402b186b3757bf2105462c3de1deae96868004073967f0005bd8c11dfe6c16` |

## 已知漏洞：历史关键句捷径

历史里与当前同一时刻的那条「等待」，对方回话只有两种：「我们没接上话」对应应该联系，「没打断我，谢谢」对应应该等待。模型不核对时间，只认这一句话就能答对。所以本轮分不清它学到的是「对上时间再判断」，还是「认关键句」。

第二轮 A1 使用早上 10 点、夜间 22 点各一次联系和可观察等待，使这四种历史回应在每块里都出现，另外用第一轮 adapter 直接考第二轮的新题。

## 缺失证据

- 训练时逐步 **loss、`rewards/accuracies`** 只打印在屏幕，没有保存文件，本轮缺。

## 审查状态

训练后指标、结果文件 SHA-256 与覆盖模块的实际名称后缀已按 PR 评论补齐；不更改第一轮脚本、不重跑训练、不覆盖已有 adapter 或冻结数据。
