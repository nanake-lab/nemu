**档位：重档（本地 GPU 实测）**

# R-BRAIN-01 环境体检与训练环境验证

- 检查日期：2026-11-16
- 分支：`r-brain/env-check`
- 工作区：`D:\Dev\nemu-r-brain`
- 创建时的 HEAD：`fc5f4c2d`

以下是第一步五条命令及原始输出。

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

## 第二步：独立训练环境验收（2026-11-16）

- 环境：`D:\Dev\cache\R-BRAIN-01\.venv`，Python 3.12.10，`pip check` 通过；版本见 [`pip-freeze.txt`](pip-freeze.txt)。
- P0-12 [inventory](../../P0-12/preparation/resume-01/model-inventory.json) 的 **14 个文件**：大小 + SHA-256 全部匹配。
- `transformers==5.23.0` 从本地加载 `Qwen3_5ForCausalLM`（NF4 4-bit）并生成中文；`missing_keys=[]`、`mismatched_keys=[]`、`loader_errors=[]`，unexpected keys 为 0。
- CUDA 峰值：allocated **3052.6 MiB**，reserved **3122.0 MiB**（单次测试读数，不是训练显存峰值）。
- 复现脚本：[ `r_brain_load_test.py`](r_brain_load_test.py)，全程离线读取本地权重；未安装 GPU 加速可选内核，推理会较慢。
- 先前脚本曾因传错 `BatchEncoding` 导致生成失败，已修复。旧环境 `D:\Dev\r-brain-train\.venv` 仍待清理；**LoRA 训练尚未验证**。
