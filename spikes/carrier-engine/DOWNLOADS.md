# 分段下载与接续

用户 2026-10-25 已授权开始下载。缓存统一放在 `D:\Dev\cache\P0-10\`，不加载模型、不运行 GPU。

## 固定来源

- 模型：Qwen/Qwen3-4B-Instruct-2507，`cdbee75f17c01a7cc42f958dc650907174af0554`；三个权重 SHA-256 与 P0-07 已提交清单逐一比较。
- 引擎档案：官方 `v0.7.0` 的 `nightly-tag.txt` 指向 `b11669`，提交 `620d2225cc436ab0b001e3200e590e3b286ef9ab`。固定 Windows x64 CUDA 12.4 主程序包及对应 runtime 包，校验 GitHub release 元数据给出的 SHA-256。不下载 CUDA toolkit 或改系统设置，不在此作生产选型。
- 模型元数据保存为 `downloads/qwen-revision-metadata.json`；引擎元数据保存为 `downloads/llama-nightly-metadata.json`，均位于上述缓存目录。

## 重启后继续

先确认上一次下载进程已结束，避免两个进程写同一分段。分别在本工作区运行：

```powershell
$env:PYTHONDONTWRITEBYTECODE = '1'
python -u spikes/carrier-engine/download_model.py --cache-root D:/Dev/cache/P0-10 --metadata D:/Dev/cache/P0-10/downloads/qwen-revision-metadata.json
python -u spikes/carrier-engine/download_engine.py --cache-root D:/Dev/cache/P0-10 --metadata D:/Dev/cache/P0-10/downloads/llama-nightly-metadata.json
```

两个入口只读复用 `workers/model-adapter/probe/download_weights.py` 的已存在分段函数，不修改它。每个大文件拆为 8 MiB，最多四个并发请求；curl 每段最多三次自动重试。重启后只有长度和 `Content-Range` 都匹配的完整分段会被复用，未完整的段重新请求，最多重传该 8 MiB。curl 的请求超时、TLS 验证均保留；不关闭证书校验。

分段与 headers 都保留在对应文件旁的 `.chunks/` 目录。headers 包含 CDN 重定向地址，只留缓存，不提交仓库。每个文件按顺序合并到 `.partial`，全部长度和上游摘要一致才改成正式文件名。任何哈希不符保留原文件与分段并停止，不静默认可或覆盖已命名的文件。

网络抖动超过单段重试次数时，再运行同一个命令即可复用已完成分段；无需删除缓存。这里的接续粒度为分段，不宣称未完成段内的字节也能接续。

## 完成证据

- 模型：`downloads/inventories/qwen-<UTC>.json`。
- 引擎：`downloads/engine/b11669/inventory-<UTC>.json`。
- 每次执行的控制台日志单独命名，不用后一次覆盖失败记录。

上述库存代表下载与完整性校验，不代表解压、转换、引擎兼容、注入或 GPU 验收通过。
