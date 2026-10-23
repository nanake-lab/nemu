# P4-01 D 盘缓存迁移 — 2026-10-21

用户明确要求迁移模型和评测环境，修改默认目录，验证运行后再清理 C 盘。新根目录为 `D:\NemuData`，位于仓库外。

| 内容 | 位置 |
| --- | --- |
| SenseVoice / Silero | `D:\NemuData\models\asr` |
| 独立 Qwen3 对齐模型 | `D:\NemuData\models\evaluation\qwen3-forced-aligner` |
| 锁定的评测虚拟环境 | `D:\NemuData\models\evaluation\alignment-venv` |
| Python 3.12.14 基础运行时 | `D:\NemuData\runtimes\python` |
| 用户录音、公开集、失败记录及冻结时标 | `D:\NemuData\asr-evaluation` |
| uv、Hugging Face、Torch 缓存及临时下载 | `D:\NemuData\cache`、`D:\NemuData\tmp` |

桌面进程、模型下载、用户登记、ASR 评测、延迟、公开集下载与 Python 独立工具使用同一根目录。Windows 默认 D 盘；显式 C 盘缓存覆盖会报错，不静默回退。可用 `NEMU_CACHE_ROOT` 指向其他非 C 盘位置。用户级 `NEMU_CACHE_ROOT`、`NEMU_ASR_MODELS`、`NEMU_ASR_DATA` 已保存为 D 盘；新启动进程继承，脚本默认不依赖该设置是否已刷新。

## 迁移与验证
- 模型、数据和基础运行时 4,497 个文件逐一核对长度和 SHA-256，全部一致，见 cache-migration-copy.json。
- 评测虚拟环境按原 uv.lock 重建；94 个包与原环境名称和版本全部一致，见 cache-migration-packages.json。解释器与所有新增下载、构建临时目录均在 D 盘。没有新增应用运行时依赖或修改模型修订、许可。
- 原生 SenseVoice 在三语公开样本各 1 条上，文本及语言结果与迁移前逐字一致；Silero 实际加载，60 条独立时标及其音频／文字哈希全部通过，见 cache-migration-native.json。
- D 盘 Python、Torch 与 qwen-asr 模块路径已检查，Qwen3 ForcedAligner 用 D 盘权重完成一次实际离线推理，见 cache-migration-alignment.json；没有重新下载模型或重新生成既有时标。
- 首次 Node 临时验证命令误用 CJS，未运行推理；改为 ESM。第二次因其他任务重负载被资源检查拒绝。原始日志保留，第三次实际验证通过。供应商 nagisa 的 SyntaxWarning 保留，未修改第三方源码。

## C 盘清理
仅在上述复制哈希、依赖版本及两种实际推理通过后，清理了 `%LOCALAPPDATA%\Nemu\models\asr`、`models\evaluation`、`asr-evaluation`。三处旧目录均已不存在，逻辑文件总量 6,061,836,956 字节；该数不是独占物理磁盘释放量，因为旧 uv 环境含共享硬链接。

普通清理遇到一个 pydantic_core 扩展映射。确认它有 13 个硬链接、D 盘同文件 SHA-256 一致后，只对旧虚拟环境目录项使用 Windows POSIX unlink；其他链接和持有句柄的工具继续保留，不结束进程。依据 [Microsoft FILE_DISPOSITION_INFORMATION_EX](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddk/ns-ntddk-_file_disposition_information_ex) 的句柄与名称空间语义。见 cache-migration-cleanup.json。

原始附件位于 D 盘，未改动。全部录音、文字稿、模型及私有详细迁移日志仍在仓库外。原已冻结的时标、模型 SHA-256 和预登记文档不变；仅改变物理目录，不把路径迁移当成新的延迟验收。此前资源窗口中断的 INVALID 结果保留；整卡延迟与最终审查仍未完成，PR #44 保持草稿和 needs-user。
