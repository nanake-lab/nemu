**档位：重档（包含 GPU 实测；新增核验未完成）**

**档位：重档**（包含 GPU 加载与显存测量；完整哈希及缺失键复测未完成）

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

## 第二步结果（2026-11-16）

- 独立 Python 3.12.10 环境；依赖导入及 `pip check` 通过。具体版本见 [`pip-freeze.txt`](pip-freeze.txt)。
- 本地 Qwen3.5-4B（图文权重）可用 `transformers==5.23.0` 加载纯文字 `Qwen3_5ForCausalLM`，NF4 4-bit 中文生成通过；无需加载视觉分支。
- CUDA 峰值：allocated **3052.8 MiB**、reserved **3122.0 MiB**。
- 首轮生成失败因测试脚本传错 `BatchEncoding`；已修复并复测成功。
- **尚未完成：** [核验脚本](r_brain_load_test.py) 要按 [P0-12 inventory](../../P0-12/preparation/resume-01/model-inventory.json) 逐文件核对大小/SHA-256，并检查 `missing_keys=0`。此前通过的加载测试不等于这两项已通过。
- **路径需整改：** 首次 venv 在 `D:\Dev\r-brain-train\.venv`，不符合 AGENTS；应在 `D:\Dev\cache\R-BRAIN-01\.venv` 重新创建和验证，成功后再处理旧环境。
- LoRA 训练尚未验证；未改系统 Python、语音环境或永久环境变量。
