# R-BRAIN-01 环境体检（只读，不安装）

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
