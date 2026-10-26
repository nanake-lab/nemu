# 本次原始证据的无损归档

原始目录：`D:/Dev/cache/P0-10/full-runs/run-01`，缓存仍保留。该次 GPU 已结束，注入执行完成，server 检查失败后停止。读取归档不加载模型或引擎。

- `summary.json`：最终离线分析；逐语言、模式和输出位置的差值/字节一致性，掩码、方向、显存及失败。
- `raw-filemap.json`：全部 6,254 个原文件的相对路径、原字节数、SHA-256 和 gzip 对象路径；所有成功及失败文件均在其中。
- `objects/`：1,115 个唯一原文件的无损 gzip。重复文件共用对象，未丢弃重复出现的位置/次数信息；映射保留全部原路径。
- `context/`：供直接查看的原始授权记录、启动日志、server 日志/退出及 native 退出；同一运行内文件也存在于对象归档。
- `analysis-checks/`：原文件哈希清单、独立 gzip 校验、独立标准库数值/哈希对照，以及 Ruff/mypy 原始结果。初始类型注解失败与修正后结果均保留。

原文件 830,513,016 bytes；唯一 gzip 对象 246,863,053 bytes。生成器对每个对象作逐字节解压比对；另一个机械 reporter 独立核验映射 6,254/6,254、压缩及展开的哈希/长度 1,115/1,115，错误 0。数据 float32 为 little-endian。非零臂的 token 表记录预测值及实际喂入的参考 token，不能当作自由生成。

以下 Python 例子只在内存读取一条原始日志，不解压到文件：

```python
from pathlib import Path
import gzip, hashlib, json

root = Path("D:/Dev/nemu-wt/P0-10/reports/P0-10/full-run-01")
mapping = json.loads((root / "raw-filemap.json").read_text(encoding="utf-8"))
item = next(x for x in mapping["files"] if x["path"] == "benchmark/Q4_K_M/server/server.stderr.log")
raw = gzip.decompress((root / item["object"]).read_bytes())
assert len(raw) == item["bytes"] and hashlib.sha256(raw).hexdigest() == item["sha256"]
print(raw.decode("utf-8"))
```

更换 `item` 的相对路径可读取任意原记录。完整原生模型加载日志为 `injection/native.stderr.log`；逐位置 logits 示例为 `injection/raw/zh/native-A/0-logits.f32`。
