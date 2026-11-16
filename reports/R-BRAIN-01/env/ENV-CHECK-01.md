**档位：重档**（包含 GPU 加载与显存测量；完整哈希及缺失键复测未完成）

# R-BRAIN-01 环境体检与训练环境验证

- 检查日期：2026-11-16
- 分支：`r-brain/env-check`
- 工作区：`D:\Dev\nemu-r-brain`
- 创建时的 HEAD：`fc5f4c2d`

以下保留五条原始命令及当时的终端输出；**不重新运行，不替换测量值**。

## 1. GPU

命令：

```powershell
nvidia-smi --query-gpu=name,memory.total,memory.used,driver_version --format=csv
```

原始输出：

```text
name, memory.total [MiB], memory.used [MiB], driver_version
NVIDIA GeForce RTX 3080, 10240 MiB, 1064 MiB, 610.88
```

## 2. Python 版本

命令：

```powershell
py -0
```

原始输出：

```text
 -V:3.14 *        Python 3.14.0
 -V:3.12          Python 3.12 (64-bit)
 -V:Astral/CPython3.12.14 CPython 3.12.14 (64-bit)
```

## 3. 当前默认 Python 的依赖

命令：

```powershell
python -m pip list 2>$null | Select-String -Pattern '^(torch|transformers|peft|trl|bitsandbytes|accelerate|datasets|safetensors) '
```

原始输出：**空（无输出）**。

## 4. 当前默认 Python 的 PyTorch / CUDA

命令：

```powershell
python -c "import torch; print(torch.__version__, torch.version.cuda, torch.cuda.is_available())"
```

原始输出：

```text
Traceback (most recent call last):
  File "<string>", line 1, in <module>
ModuleNotFoundError: No module named 'torch'
```

## 5. D 盘可用空间

命令：

```powershell
Get-PSDrive D | Select-Object Used,Free
```

原始输出：

```text
        Used        Free
        ----        ----
645660499968 92398587904
```

## 补充只读检查（原任务五项之外）

`py -3.12 -m pip list` 使用同一依赖过滤条件：无输出。

找到多个独立虚拟环境；其中检查了：

- `D:\Dev\cache\p4-02\inference-venv`：尝试导入 `torch`，报错 `ModuleNotFoundError: No module named 'torch'`。
- `D:\Dev\cache\p4-02\gsv-v5-venv`：通过直接执行该环境 Python 验证，得到以下原始输出：

```text
2.10.0+cu128 12.8 True
```

该虚拟环境执行 `python -m pip list` 的原始错误：

```text
D:\Dev\cache\p4-02\gsv-v5-venv\Scripts\python.exe: No module named pip
```

使用 `importlib.metadata` 检查的原始输出：

```text
torch 2.10.0+cu128
transformers 4.57.3
peft 0.17.1
trl 未安装
bitsandbytes 未安装
accelerate 1.15.0
datasets 未安装
safetensors 0.8.0
```

## 结论

- RTX 3080（10 GiB 显存）可检测到；驱动 `610.88`。
- 当前默认 Python 无法导入 `torch`，但已存在的 GSV-V5 语音虚拟环境可使用 CUDA。
- 训练依赖不完整；本次**不安装、不升级、不修改现有环境**。未来如需训练，建议单独建立环境，避免影响语音系统。
- 本次仅保存观察结果；未进行训练，也未验证该硬件上具体训练方案的可行性。

## 第二步：训练环境与 Qwen3.5-4B 本地验证（2026-11-16）

- Python 3.12.10；全部训练包可导入，`pip check` 无冲突；PyTorch `2.10.0+cu128`，CUDA 12.8 可用。
- 实测 `transformers==5.23.0` + `bitsandbytes==0.50.2`：Qwen3.5-4B 原始图文权重的**纯文字部分**可用 `Qwen3_5ForCausalLM` 以 NF4 4-bit 加载并生成中文。
- 成功测试的 CUDA 峰值：allocated **3052.8 MiB**，reserved **3122.0 MiB**。证据：[`QWEN-4BIT-CHECK-02.txt`](QWEN-4BIT-CHECK-02.txt)；全量依赖：[ `pip-freeze.txt`](pip-freeze.txt)。
- 首次生成失败是测试脚本误传 `BatchEncoding` 导致；修正传入 `input_ids` 后通过。不是权重/显存失败，失败日志未纳入 PR。
- **路径待纠正：** 首次安装在 `D:\Dev\r-brain-train\.venv`，不符合 AGENTS 规定；须按版本清单重新建立在 `D:\Dev\cache\R-BRAIN-01\.venv`，不直接移动已有 venv。新环境复测前不认定整改完成。
- **待实测：** 依据 P0-12 [inventory](../../P0-12/preparation/resume-01/model-inventory.json) 全量核对大小和 SHA-256；加载时输出 `missing_keys`（必须为 0）、`mismatched_keys` 与意外非视觉权重。已新增可复现脚本 [`r_brain_load_test.py`](r_brain_load_test.py)，执行通过后再更新结论。
- 此处只验证推理加载，不代表 LoRA 训练已通过。未改动系统 Python、语音环境或系统/用户级环境变量。
