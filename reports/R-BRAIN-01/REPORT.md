**档位：重档（Windows RTX 3080 GPU 实测）**

# R-BRAIN-01：DPO / QLoRA 最小训练验证

**结论：PASS。** 同一台 RTX 3080（10GB）用本地 Qwen3.5-4B NF4 4-bit 基座、TRL DPO + PEFT LoRA，完成 256 与 512 两档各 10 步训练。梯度和 LoRA B 权重均非零；adapter 保存后可重载生成文字；原始 safetensors 在训练前后 SHA-256 一致。这里只证明这套最小配置能训练，不评价学到了什么。

## 实测对比

| 指标 | 256 tokens | 512 tokens |
|---|---:|---:|
| 实际 batch 形状（chosen + rejected） | 2 × 256 | 2 × 512 |
| 有效 token 数（示例 batch） | 234, 233 | 484, 483 |
| 完成步数 | 10/10 | 10/10 |
| 第 1 步 | 2.23 s | 2.63 s |
| 第 2–10 步平均 | 0.97 s | 1.59 s |
| 训练总运行时间（Trainer） | 11 s | 16.93 s |
| CUDA allocated 峰值 | 3584.70 MiB | 4184.80 MiB |
| CUDA reserved 峰值 | 3700.00 MiB | 4486.00 MiB |
| LoRA B 终态范数（最小～最大） | 0.0021565999～0.0065531959 | 0.0021342244～0.0065311836 |
| loss / grad_norm | 全部有限 | 全部有限 |
| 本地 adapter 保存 + 重载生成 | PASS | PASS |
| 原始权重训练前/后 SHA-256 | 一致 | 一致 |

512 对比 256，**后 9 步平均耗时 +63%**，CUDA reserved 峰值 **+786.00 MiB**。本表只代表当前少量短句、合成长度占位的实测配置。

## 设置与范围

- **模型**：P0-12 本地已核验的 `Qwen3.5-4B` 原始 safetensors；只加载 `Qwen3_5ForCausalLM` 文字分支。离线，不下载模型。
- **版本**：torch 2.10.0+cu128；transformers 5.23.0；trl 1.18.0；peft 0.21.3。其他锁定版本见上一步 `pip-freeze.txt`。
- **数据**：脚本中 4 对通用中文偏好短句，长度档用中性文本填充。无真实个人经历数据。
- **训练**：batch 1、10 步/档、梯度检查点、bf16、AdamW、LR 5e-6、DPO beta 0.1、seed 17；LoRA r=8、alpha=16、dropout=0，仅 q_proj/v_proj。参考模型：同一基座禁用 adapter，`ref_model=None`，未加载第二份。
- **实际 LoRA 覆盖**：完整注意力层 **3、7、11、15、19、23、27、31** 各 q_proj 和 v_proj，共 **16 个模块、32 个可训练 LoRA 张量**；没有覆盖 Gated DeltaNet 的线性注意力投影。
- **慢速基线**：未安装 `causal_conv1d` 或 `flash-linear-attention`，对应计算回退至 PyTorch 实现。测量时 Windows Runner 已停止。
- **adapter**：`D:\Dev\cache\R-BRAIN-01\dpo\adapter-256`、`adapter-512`，不提交仓库。

**失败与未完成**：两档没有 OOM/NaN/Inf。此实验不评估偏好学习质量、其他 LoRA 层覆盖、正式训练时长或训练数据的泛化表现，也没有安装加速内核。没有修改原始权重文件。

## 原始终端输出

以下两段分别来自用户在 Windows 上的实际运行，保留原始命令、警告、逐步日志和结果，未加工数值。复现脚本：[`dpo/dpo_smoke.py`](dpo/dpo_smoke.py)，运行时为审查通过的 head `49cf42f5`。

### 长度 256

```text
PS D:\Dev\nemu-r-brain> & "D:\Dev\cache\R-BRAIN-01\.venv\Scripts\python.exe" "D:\Dev\nemu-wt\R-BRAIN-01-DPO\reports\R-BRAIN-01\dpo\dpo_smoke.py" --length 256
W1116 21:21:27.631000 12936 Lib\site-packages\torch\distributed\elastic\multiprocessing\redirects.py:29] NOTE: Redirects are currently not supported in Windows or MacOs.
Runner check: PASS (Listener and Worker not running)
length=256 steps=10 batch=1 seed=17 LoRA(r=8,alpha=16,dropout=0,targets=q_proj/v_proj)
optimizer=adamw_torch lr=5e-6 beta=0.1 grad_checkpoint=True
ref_model=None (use base with adapter disabled); model files: LOCAL ONLY
versions torch=2.10.0+cu128 transformers=5.23.0 trl=1.18.0 peft=0.21.3
BEFORE SHA-256 OK: model.safetensors-00001-of-00002.safetensors 26a93f066e1916adb13453dae5a0c707c0fbc71299ed98779571a907b8e74c61
BEFORE SHA-256 OK: model.safetensors-00002-of-00002.safetensors cb544bd9bfae93dc59b0f22b292f5933573854a7f9b97835c67060d7d910e188
Loading weights: 100%|███████████████████████████████████████████████████████████████| 426/426 [00:08<00:00, 49.44it/s]
D:\Dev\cache\R-BRAIN-01\.venv\Lib\site-packages\transformers\training_args.py:1541: FutureWarning: `gradient_checkpointing` is deprecated and will be removed in v6. Use `activation_checkpointing` instead.
  warnings.warn(f"`{old}` is deprecated and will be removed in v6. Use `{new}` instead.", FutureWarning)
D:\Dev\cache\R-BRAIN-01\.venv\Lib\site-packages\transformers\training_args.py:1541: FutureWarning: `gradient_checkpointing_kwargs` is deprecated and will be removed in v6. Use `activation_checkpointing_kwargs` instead.
  warnings.warn(f"`{old}` is deprecated and will be removed in v6. Use `{new}` instead.", FutureWarning)
D:\Dev\cache\R-BRAIN-01\.venv\Lib\site-packages\transformers\modeling_utils.py:3303: FutureWarning: `is_gradient_checkpointing` is deprecated and will be removed in v6. Use `is_activation_checkpointing` instead.
  warnings.warn(
Adding EOS to train dataset: 100%|███████████████████████████████████████████████| 4/4 [00:00<00:00, 379.31 examples/s]
Tokenizing train dataset: 100%|██████████████████████████████████████████████████| 4/4 [00:00<00:00, 468.06 examples/s]
Dropping fully truncated examples from train dataset: 100%|██████████████████████| 4/4 [00:00<00:00, 444.46 examples/s]
LoRA trainable tensors: 32
LoRA module: layer=3 kind=full-attention path=base_model.model.model.layers.3.self_attn.q_proj base_type=Linear4bit
LoRA module: layer=3 kind=full-attention path=base_model.model.model.layers.3.self_attn.v_proj base_type=Linear4bit
LoRA module: layer=7 kind=full-attention path=base_model.model.model.layers.7.self_attn.q_proj base_type=Linear4bit
LoRA module: layer=7 kind=full-attention path=base_model.model.model.layers.7.self_attn.v_proj base_type=Linear4bit
LoRA module: layer=11 kind=full-attention path=base_model.model.model.layers.11.self_attn.q_proj base_type=Linear4bit
LoRA module: layer=11 kind=full-attention path=base_model.model.model.layers.11.self_attn.v_proj base_type=Linear4bit
LoRA module: layer=15 kind=full-attention path=base_model.model.model.layers.15.self_attn.q_proj base_type=Linear4bit
LoRA module: layer=15 kind=full-attention path=base_model.model.model.layers.15.self_attn.v_proj base_type=Linear4bit
LoRA module: layer=19 kind=full-attention path=base_model.model.model.layers.19.self_attn.q_proj base_type=Linear4bit
LoRA module: layer=19 kind=full-attention path=base_model.model.model.layers.19.self_attn.v_proj base_type=Linear4bit
LoRA module: layer=23 kind=full-attention path=base_model.model.model.layers.23.self_attn.q_proj base_type=Linear4bit
LoRA module: layer=23 kind=full-attention path=base_model.model.model.layers.23.self_attn.v_proj base_type=Linear4bit
LoRA module: layer=27 kind=full-attention path=base_model.model.model.layers.27.self_attn.q_proj base_type=Linear4bit
LoRA module: layer=27 kind=full-attention path=base_model.model.model.layers.27.self_attn.v_proj base_type=Linear4bit
LoRA module: layer=31 kind=full-attention path=base_model.model.model.layers.31.self_attn.q_proj base_type=Linear4bit
LoRA module: layer=31 kind=full-attention path=base_model.model.model.layers.31.self_attn.v_proj base_type=Linear4bit
LoRA coverage: count= 16 layers_by_type= {'full-attention': ['11', '15', '19', '23', '27', '3', '31', '7']}
LoRA B initial max norm: 0
TRL batch keys: ['attention_mask', 'completion_mask', 'input_ids']
Batch input shape=(2, 256) valid_tokens=[234, 233] completion_tokens=[8, 7]
[transformers] The tokenizer has new PAD/BOS/EOS tokens that differ from the model config and generation config. The model config and generation config were aligned accordingly, being updated with the tokenizer's values. Updated tokens: {'eos_token_id': 248046}.
[transformers] `causal_conv1d_fn` is falling back to its reference PyTorch implementation because `causal_conv1d` is not installed. This is correct but much slower; install `causal_conv1d` for the optimized kernel.
[transformers] `chunk_gated_delta_rule` is falling back to its reference PyTorch implementation because `flash-linear-attention` is not installed. This is correct but much slower; install `flash-linear-attention` for the optimized kernel.
step=01 sec=2.234 peak_alloc_MiB=3581.2 peak_reserved_MiB=3696.0 loss=0.6931472 grad_norm=1.132812
{'loss': '0.6931', 'grad_norm': '1.133', 'learning_rate': '5e-06', 'entropy': '2.849', 'num_tokens': '467', 'logits/chosen': '-1.417', 'logits/rejected': '-1.332', 'mean_token_accuracy': '0.25', 'rewards/chosen': '0', 'rewards/rejected': '0', 'rewards/accuracies': '0', 'rewards/margins': '0', 'logps/chosen': '-26.59', 'logps/rejected': '-25.96', 'epoch': '0.25'}
step=02 sec=0.999 peak_alloc_MiB=3584.7 peak_reserved_MiB=3700.0 loss=0.7065322 grad_norm=1.726562
{'loss': '0.7065', 'grad_norm': '1.727', 'learning_rate': '5e-06', 'entropy': '2.893', 'num_tokens': '938', 'logits/chosen': '-1.31', 'logits/rejected': '-1.58', 'mean_token_accuracy': '0.25', 'rewards/chosen': '-0.001492', 'rewards/rejected': '0.0251', 'rewards/accuracies': '0', 'rewards/margins': '-0.02659', 'logps/chosen': '-26.06', 'logps/rejected': '-29.89', 'epoch': '0.5'}
step=03 sec=0.947 peak_alloc_MiB=3584.7 peak_reserved_MiB=3700.0 loss=0.6981665 grad_norm=1.710938
{'loss': '0.6982', 'grad_norm': '1.711', 'learning_rate': '5e-06', 'entropy': '3.091', 'num_tokens': '1405', 'logits/chosen': '-1.458', 'logits/rejected': '-1.961', 'mean_token_accuracy': '0.125', 'rewards/chosen': '-0.001389', 'rewards/rejected': '0.008625', 'rewards/accuracies': '0', 'rewards/margins': '-0.01001', 'logps/chosen': '-20.33', 'logps/rejected': '-34.78', 'epoch': '0.75'}
step=04 sec=0.904 peak_alloc_MiB=3584.7 peak_reserved_MiB=3700.0 loss=0.6914766 grad_norm=1.21875
{'loss': '0.6915', 'grad_norm': '1.219', 'learning_rate': '5e-06', 'entropy': '3.362', 'num_tokens': '1871', 'logits/chosen': '-1.389', 'logits/rejected': '-1.616', 'mean_token_accuracy': '0.2222', 'rewards/chosen': '0.01929', 'rewards/rejected': '0.01594', 'rewards/accuracies': '1', 'rewards/margins': '0.003344', 'logps/chosen': '-33.36', 'logps/rejected': '-32.46', 'epoch': '1'}
step=05 sec=1.011 peak_alloc_MiB=3584.7 peak_reserved_MiB=3700.0 loss=0.6914418 grad_norm=1.210938
{'loss': '0.6914', 'grad_norm': '1.211', 'learning_rate': '5e-06', 'entropy': '3.377', 'num_tokens': '2337', 'logits/chosen': '-1.394', 'logits/rejected': '-1.613', 'mean_token_accuracy': '0.2222', 'rewards/chosen': '0.002562', 'rewards/rejected': '-0.0008518', 'rewards/accuracies': '1', 'rewards/margins': '0.003414', 'logps/chosen': '-33.53', 'logps/rejected': '-32.62', 'epoch': '1.25'}
step=06 sec=0.989 peak_alloc_MiB=3584.7 peak_reserved_MiB=3700.0 loss=0.7078821 grad_norm=1.703125
{'loss': '0.7079', 'grad_norm': '1.703', 'learning_rate': '5e-06', 'entropy': '3.074', 'num_tokens': '2804', 'logits/chosen': '-1.453', 'logits/rejected': '-1.957', 'mean_token_accuracy': '0.125', 'rewards/chosen': '-0.004395', 'rewards/rejected': '0.02486', 'rewards/accuracies': '0', 'rewards/margins': '-0.02926', 'logps/chosen': '-20.36', 'logps/rejected': '-34.62', 'epoch': '1.5'}
step=07 sec=0.993 peak_alloc_MiB=3584.7 peak_reserved_MiB=3700.0 loss=0.7213385 grad_norm=1.710938
{'loss': '0.7213', 'grad_norm': '1.711', 'learning_rate': '5e-06', 'entropy': '2.891', 'num_tokens': '3275', 'logits/chosen': '-1.308', 'logits/rejected': '-1.578', 'mean_token_accuracy': '0.25', 'rewards/chosen': '-0.01428', 'rewards/rejected': '0.04133', 'rewards/accuracies': '0', 'rewards/margins': '-0.05561', 'logps/chosen': '-26.19', 'logps/rejected': '-29.73', 'epoch': '1.75'}
step=08 sec=0.972 peak_alloc_MiB=3584.7 peak_reserved_MiB=3700.0 loss=0.7079717 grad_norm=1.15625
{'loss': '0.708', 'grad_norm': '1.156', 'learning_rate': '5e-06', 'entropy': '2.841', 'num_tokens': '3742', 'logits/chosen': '-1.413', 'logits/rejected': '-1.336', 'mean_token_accuracy': '0.25', 'rewards/chosen': '-0.01046', 'rewards/rejected': '0.01897', 'rewards/accuracies': '0', 'rewards/margins': '-0.02943', 'logps/chosen': '-26.69', 'logps/rejected': '-25.77', 'epoch': '2'}
step=09 sec=0.919 peak_alloc_MiB=3584.7 peak_reserved_MiB=3700.0 loss=0.7082905 grad_norm=1.710938
{'loss': '0.7083', 'grad_norm': '1.711', 'learning_rate': '5e-06', 'entropy': '2.88', 'num_tokens': '4213', 'logits/chosen': '-1.311', 'logits/rejected': '-1.579', 'mean_token_accuracy': '0.25', 'rewards/chosen': '-0.009758', 'rewards/rejected': '0.0203', 'rewards/accuracies': '0', 'rewards/margins': '-0.03006', 'logps/chosen': '-26.15', 'logps/rejected': '-29.94', 'epoch': '2.25'}
step=10 sec=1.009 peak_alloc_MiB=3584.7 peak_reserved_MiB=3700.0 loss=0.6971993 grad_norm=1.265625
{'loss': '0.6972', 'grad_norm': '1.266', 'learning_rate': '5e-06', 'entropy': '3.372', 'num_tokens': '4679', 'logits/chosen': '-1.39', 'logits/rejected': '-1.616', 'mean_token_accuracy': '0.2222', 'rewards/chosen': '0.001723', 'rewards/rejected': '0.009811', 'rewards/accuracies': '0', 'rewards/margins': '-0.008088', 'logps/chosen': '-33.53', 'logps/rejected': '-32.52', 'epoch': '2.5'}
{'train_runtime': '11', 'train_samples_per_second': '0.909', 'train_steps_per_second': '0.909', 'train_loss': '0.7023', 'epoch': '2.5'}
LoRA B final norms: count=16 min=0.0021565999 max=0.0065531959
Training steps: PASS 10; maximum CUDA reserved MiB: 3700.0
Adapter saved: D:\Dev\cache\R-BRAIN-01\dpo\adapter-256
Loading weights: 100%|███████████████████████████████████████████████████████████████| 426/426 [00:09<00:00, 46.54it/s]
[transformers] `causal_conv1d_update` is falling back to its reference PyTorch implementation because `causal_conv1d` is not installed. This is correct but much slower; install `causal_conv1d` for the optimized kernel.
[transformers] `fused_recurrent_gated_delta_rule` is falling back to its reference PyTorch implementation because `flash-linear-attention` is not installed. This is correct but much slower; install `flash-linear-attention` for the optimized kernel.
Reloaded base + adapter generated: '你好！我是 Qwen3.5，阿里巴巴最新推出的通义千问大语言模型。我具备强大的语言理解、逻辑推理、代码生成及'
AFTER SHA-256 OK: model.safetensors-00001-of-00002.safetensors 26a93f066e1916adb13453dae5a0c707c0fbc71299ed98779571a907b8e74c61
AFTER SHA-256 OK: model.safetensors-00002-of-00002.safetensors cb544bd9bfae93dc59b0f22b292f5933573854a7f9b97835c67060d7d910e188
RESULT length=256: PASS
```

### 长度 512

```text
PS D:\Dev\nemu-r-brain> & "D:\Dev\cache\R-BRAIN-01\.venv\Scripts\python.exe" "D:\Dev\nemu-wt\R-BRAIN-01-DPO\reports\R-BRAIN-01\dpo\dpo_smoke.py" --length 512
W1116 21:24:27.878000 4444 Lib\site-packages\torch\distributed\elastic\multiprocessing\redirects.py:29] NOTE: Redirects are currently not supported in Windows or MacOs.
Runner check: PASS (Listener and Worker not running)
length=512 steps=10 batch=1 seed=17 LoRA(r=8,alpha=16,dropout=0,targets=q_proj/v_proj)
optimizer=adamw_torch lr=5e-6 beta=0.1 grad_checkpoint=True
ref_model=None (use base with adapter disabled); model files: LOCAL ONLY
versions torch=2.10.0+cu128 transformers=5.23.0 trl=1.18.0 peft=0.21.3
BEFORE SHA-256 OK: model.safetensors-00001-of-00002.safetensors 26a93f066e1916adb13453dae5a0c707c0fbc71299ed98779571a907b8e74c61
BEFORE SHA-256 OK: model.safetensors-00002-of-00002.safetensors cb544bd9bfae93dc59b0f22b292f5933573854a7f9b97835c67060d7d910e188
Loading weights: 100%|███████████████████████████████████████████████████████████████| 426/426 [00:08<00:00, 48.06it/s]
D:\Dev\cache\R-BRAIN-01\.venv\Lib\site-packages\transformers\training_args.py:1541: FutureWarning: `gradient_checkpointing` is deprecated and will be removed in v6. Use `activation_checkpointing` instead.
  warnings.warn(f"`{old}` is deprecated and will be removed in v6. Use `{new}` instead.", FutureWarning)
D:\Dev\cache\R-BRAIN-01\.venv\Lib\site-packages\transformers\training_args.py:1541: FutureWarning: `gradient_checkpointing_kwargs` is deprecated and will be removed in v6. Use `activation_checkpointing_kwargs` instead.
  warnings.warn(f"`{old}` is deprecated and will be removed in v6. Use `{new}` instead.", FutureWarning)
D:\Dev\cache\R-BRAIN-01\.venv\Lib\site-packages\transformers\modeling_utils.py:3303: FutureWarning: `is_gradient_checkpointing` is deprecated and will be removed in v6. Use `is_activation_checkpointing` instead.
  warnings.warn(
Adding EOS to train dataset: 100%|███████████████████████████████████████████████| 4/4 [00:00<00:00, 368.32 examples/s]
Tokenizing train dataset: 100%|██████████████████████████████████████████████████| 4/4 [00:00<00:00, 204.57 examples/s]
Dropping fully truncated examples from train dataset: 100%|███████████████████████| 4/4 [00:00<00:00, 98.92 examples/s]
LoRA trainable tensors: 32
LoRA module: layer=3 kind=full-attention path=base_model.model.model.layers.3.self_attn.q_proj base_type=Linear4bit
LoRA module: layer=3 kind=full-attention path=base_model.model.model.layers.3.self_attn.v_proj base_type=Linear4bit
LoRA module: layer=7 kind=full-attention path=base_model.model.model.layers.7.self_attn.q_proj base_type=Linear4bit
LoRA module: layer=7 kind=full-attention path=base_model.model.model.layers.7.self_attn.v_proj base_type=Linear4bit
LoRA module: layer=11 kind=full-attention path=base_model.model.model.layers.11.self_attn.q_proj base_type=Linear4bit
LoRA module: layer=11 kind=full-attention path=base_model.model.model.layers.11.self_attn.v_proj base_type=Linear4bit
LoRA module: layer=15 kind=full-attention path=base_model.model.model.layers.15.self_attn.q_proj base_type=Linear4bit
LoRA module: layer=15 kind=full-attention path=base_model.model.model.layers.15.self_attn.v_proj base_type=Linear4bit
LoRA module: layer=19 kind=full-attention path=base_model.model.model.layers.19.self_attn.q_proj base_type=Linear4bit
LoRA module: layer=19 kind=full-attention path=base_model.model.model.layers.19.self_attn.v_proj base_type=Linear4bit
LoRA module: layer=23 kind=full-attention path=base_model.model.model.layers.23.self_attn.q_proj base_type=Linear4bit
LoRA module: layer=23 kind=full-attention path=base_model.model.model.layers.23.self_attn.v_proj base_type=Linear4bit
LoRA module: layer=27 kind=full-attention path=base_model.model.model.layers.27.self_attn.q_proj base_type=Linear4bit
LoRA module: layer=27 kind=full-attention path=base_model.model.model.layers.27.self_attn.v_proj base_type=Linear4bit
LoRA module: layer=31 kind=full-attention path=base_model.model.model.layers.31.self_attn.q_proj base_type=Linear4bit
LoRA module: layer=31 kind=full-attention path=base_model.model.model.layers.31.self_attn.v_proj base_type=Linear4bit
LoRA coverage: count= 16 layers_by_type= {'full-attention': ['11', '15', '19', '23', '27', '3', '31', '7']}
LoRA B initial max norm: 0
TRL batch keys: ['attention_mask', 'completion_mask', 'input_ids']
Batch input shape=(2, 512) valid_tokens=[484, 483] completion_tokens=[8, 7]
[transformers] The tokenizer has new PAD/BOS/EOS tokens that differ from the model config and generation config. The model config and generation config were aligned accordingly, being updated with the tokenizer's values. Updated tokens: {'eos_token_id': 248046}.
[transformers] `causal_conv1d_fn` is falling back to its reference PyTorch implementation because `causal_conv1d` is not installed. This is correct but much slower; install `causal_conv1d` for the optimized kernel.
[transformers] `chunk_gated_delta_rule` is falling back to its reference PyTorch implementation because `flash-linear-attention` is not installed. This is correct but much slower; install `flash-linear-attention` for the optimized kernel.
step=01 sec=2.625 peak_alloc_MiB=4180.3 peak_reserved_MiB=4482.0 loss=0.6931472 grad_norm=1.226562
{'loss': '0.6931', 'grad_norm': '1.227', 'learning_rate': '5e-06', 'entropy': '2.857', 'num_tokens': '967', 'logits/chosen': '-1.436', 'logits/rejected': '-1.352', 'mean_token_accuracy': '0.25', 'rewards/chosen': '0', 'rewards/rejected': '0', 'rewards/accuracies': '0', 'rewards/margins': '0', 'logps/chosen': '-26.54', 'logps/rejected': '-25.55', 'epoch': '0.25'}
step=02 sec=1.608 peak_alloc_MiB=4184.8 peak_reserved_MiB=4486.0 loss=0.6923116 grad_norm=1.828125
{'loss': '0.6923', 'grad_norm': '1.828', 'learning_rate': '5e-06', 'entropy': '2.932', 'num_tokens': '1938', 'logits/chosen': '-1.318', 'logits/rejected': '-1.609', 'mean_token_accuracy': '0.25', 'rewards/chosen': '0.003725', 'rewards/rejected': '0.002053', 'rewards/accuracies': '1', 'rewards/margins': '0.001672', 'logps/chosen': '-25.6', 'logps/rejected': '-30.46', 'epoch': '0.5'}
step=03 sec=1.607 peak_alloc_MiB=4184.8 peak_reserved_MiB=4486.0 loss=0.6983194 grad_norm=1.703125
{'loss': '0.6983', 'grad_norm': '1.703', 'learning_rate': '5e-06', 'entropy': '3.101', 'num_tokens': '2905', 'logits/chosen': '-1.484', 'logits/rejected': '-1.99', 'mean_token_accuracy': '0.125', 'rewards/chosen': '-0.01349', 'rewards/rejected': '-0.003176', 'rewards/accuracies': '0', 'rewards/margins': '-0.01032', 'logps/chosen': '-20.15', 'logps/rejected': '-34.5', 'epoch': '0.75'}
step=04 sec=1.549 peak_alloc_MiB=4184.8 peak_reserved_MiB=4486.0 loss=0.6943876 grad_norm=1.523438
{'loss': '0.6944', 'grad_norm': '1.523', 'learning_rate': '5e-06', 'entropy': '3.404', 'num_tokens': '3871', 'logits/chosen': '-1.423', 'logits/rejected': '-1.625', 'mean_token_accuracy': '0.2222', 'rewards/chosen': '-0.001031', 'rewards/rejected': '0.001448', 'rewards/accuracies': '0', 'rewards/margins': '-0.002479', 'logps/chosen': '-33.49', 'logps/rejected': '-32.24', 'epoch': '1'}
step=05 sec=1.608 peak_alloc_MiB=4184.8 peak_reserved_MiB=4486.0 loss=0.7006364 grad_norm=1.945312
{'loss': '0.7006', 'grad_norm': '1.945', 'learning_rate': '5e-06', 'entropy': '3.404', 'num_tokens': '4837', 'logits/chosen': '-1.428', 'logits/rejected': '-1.636', 'mean_token_accuracy': '0.2222', 'rewards/chosen': '-0.01159', 'rewards/rejected': '0.003336', 'rewards/accuracies': '0', 'rewards/margins': '-0.01492', 'logps/chosen': '-33.6', 'logps/rejected': '-32.22', 'epoch': '1.25'}
step=06 sec=1.576 peak_alloc_MiB=4184.8 peak_reserved_MiB=4486.0 loss=0.7035711 grad_norm=1.75
{'loss': '0.7036', 'grad_norm': '1.75', 'learning_rate': '5e-06', 'entropy': '3.109', 'num_tokens': '5804', 'logits/chosen': '-1.481', 'logits/rejected': '-1.987', 'mean_token_accuracy': '0.125', 'rewards/chosen': '-0.01693', 'rewards/rejected': '0.003807', 'rewards/accuracies': '0', 'rewards/margins': '-0.02074', 'logps/chosen': '-20.18', 'logps/rejected': '-34.43', 'epoch': '1.5'}
step=07 sec=1.553 peak_alloc_MiB=4184.8 peak_reserved_MiB=4486.0 loss=0.6993292 grad_norm=1.65625
{'loss': '0.6993', 'grad_norm': '1.656', 'learning_rate': '5e-06', 'entropy': '2.923', 'num_tokens': '6775', 'logits/chosen': '-1.319', 'logits/rejected': '-1.61', 'mean_token_accuracy': '0.25', 'rewards/chosen': '-0.009769', 'rewards/rejected': '0.002557', 'rewards/accuracies': '0', 'rewards/margins': '-0.01233', 'logps/chosen': '-25.73', 'logps/rejected': '-30.45', 'epoch': '1.75'}
step=08 sec=1.557 peak_alloc_MiB=4184.8 peak_reserved_MiB=4486.0 loss=0.6871303 grad_norm=1.34375
{'loss': '0.6871', 'grad_norm': '1.344', 'learning_rate': '5e-06', 'entropy': '2.871', 'num_tokens': '7742', 'logits/chosen': '-1.43', 'logits/rejected': '-1.354', 'mean_token_accuracy': '0.25', 'rewards/chosen': '0.01113', 'rewards/rejected': '-0.0009354', 'rewards/accuracies': '1', 'rewards/margins': '0.01207', 'logps/chosen': '-26.42', 'logps/rejected': '-25.56', 'epoch': '2'}
step=09 sec=1.670 peak_alloc_MiB=4184.8 peak_reserved_MiB=4486.0 loss=0.7074974 grad_norm=1.890625
{'loss': '0.7075', 'grad_norm': '1.891', 'learning_rate': '5e-06', 'entropy': '2.931', 'num_tokens': '8713', 'logits/chosen': '-1.32', 'logits/rejected': '-1.607', 'mean_token_accuracy': '0.25', 'rewards/chosen': '-0.02305', 'rewards/rejected': '0.005452', 'rewards/accuracies': '0', 'rewards/margins': '-0.0285', 'logps/chosen': '-25.86', 'logps/rejected': '-30.42', 'epoch': '2.25'}
step=10 sec=1.555 peak_alloc_MiB=4184.8 peak_reserved_MiB=4486.0 loss=0.6980892 grad_norm=1.492188
{'loss': '0.6981', 'grad_norm': '1.492', 'learning_rate': '5e-06', 'entropy': '3.401', 'num_tokens': '9679', 'logits/chosen': '-1.423', 'logits/rejected': '-1.626', 'mean_token_accuracy': '0.2222', 'rewards/chosen': '-0.01866', 'rewards/rejected': '-0.008802', 'rewards/accuracies': '0', 'rewards/margins': '-0.00986', 'logps/chosen': '-33.67', 'logps/rejected': '-32.34', 'epoch': '2.5'}
{'train_runtime': '16.93', 'train_samples_per_second': '0.591', 'train_steps_per_second': '0.591', 'train_loss': '0.6974', 'epoch': '2.5'}
LoRA B final norms: count=16 min=0.0021342244 max=0.0065311836
Training steps: PASS 10; maximum CUDA reserved MiB: 4486.0
Adapter saved: D:\Dev\cache\R-BRAIN-01\dpo\adapter-512
Loading weights: 100%|███████████████████████████████████████████████████████████████| 426/426 [00:08<00:00, 53.23it/s]
[transformers] `causal_conv1d_update` is falling back to its reference PyTorch implementation because `causal_conv1d` is not installed. This is correct but much slower; install `causal_conv1d` for the optimized kernel.
[transformers] `fused_recurrent_gated_delta_rule` is falling back to its reference PyTorch implementation because `flash-linear-attention` is not installed. This is correct but much slower; install `flash-linear-attention` for the optimized kernel.
Reloaded base + adapter generated: '你好！我是 Qwen3.5，阿里巴巴最新推出的通义千问大语言模型。我具备强大的语言理解与生成能力，支持全球 '
AFTER SHA-256 OK: model.safetensors-00001-of-00002.safetensors 26a93f066e1916adb13453dae5a0c707c0fbc71299ed98779571a907b8e74c61
AFTER SHA-256 OK: model.safetensors-00002-of-00002.safetensors cb544bd9bfae93dc59b0f22b292f5933573854a7f9b97835c67060d7d910e188
RESULT length=512: PASS
```
