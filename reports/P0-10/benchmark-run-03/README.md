# run-03 原始证据

实际 GPU 提交 `a1f18b0f953a0291dd3f09701d35662cca436402`，Q4→Q8，18:38:24–19:58:29 Perth。948 completion（8 预热、940 正式）全部完成，父进程退出 0，注入未重跑。

- `raw-filemap.json` 与 `objects/`：3775 个原文件、138579801 bytes，以 SHA-256 去重为3152个无损gzip对象、14077348 bytes。缓存原文件未删改。
- `context/`：启动/提交绑定、退出、父日志、82次资源巡检、原文件inventory、归档日志、复制哈希。
- `analysis/summary.json`、`method-v02.py`：完整统计、失败ID、显存、计数差异及方法；不加载模型或GPU。
- `analysis/attempt-01/`：主代理初版新增分析的错误等计数假设、失败日志及差异；不是原测量判据失败，没有修改GPU记录。
- `audit/`：机械reporter独立逐条校验、输入哈希、原日志、命令和UTC。首版含IRL预热的453条统计保留，另存正式450条投影；两次main同步后检查14个实现Python及6项unittest。审计脚本超过400行，用于保留完整机械复核方法，不进入应用或worker。`verify-gzip.ps1`另用.NET解压核对3152个对象及3775个映射。

按原路径读取单个文件：

```python
from pathlib import Path
import gzip, hashlib, json
root = Path("reports/P0-10/benchmark-run-03")
mapping = json.loads((root / "raw-filemap.json").read_text(encoding="utf-8"))
entry = next(f for f in mapping["files"] if f["path"] == "Q4_K_M/requests/case-0030-wire/events.jsonl")
raw = gzip.decompress((root / entry["object"]).read_bytes())
assert len(raw) == entry["bytes"]
assert hashlib.sha256(raw).hexdigest() == entry["sha256"]
print(raw.decode("utf-8"))
```

32条SSE token ID序列不完整，未补造ID/时间，详见上级 `BENCHMARK-REPORT.md`。Schema或来源检查不能等同人工语义正确率。
