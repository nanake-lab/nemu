"""Primary-authored, read-only recipe for the routine reporter trial."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import statistics
import subprocess
import tomllib

root = Path(__file__).resolve().parents[3]
print("START_UTC=" + datetime.now(timezone.utc).isoformat(), flush=True)
config_path = root / ".codex/agents/routine-reporter.toml"
config_bytes = config_path.read_bytes()
config = tomllib.loads(config_bytes.decode("utf-8"))
old = subprocess.check_output(["git", "show", "HEAD:AGENTS.md"], cwd=root).decode("utf-8")
current = (root / "AGENTS.md").read_text(encoding="utf-8")
prefix = current.replace("\r\n", "\n").startswith(old.replace("\r\n", "\n"))
rows = []
matrix = root / "reports/P0-07/matrix-v1"
for record in json.loads((matrix / "matrix.json").read_text(encoding="utf-8")):
    case = record["model"].replace("/", "--") + "-" + record["precision"]
    path = matrix / case / "concurrent.json"
    data = json.loads(path.read_text(encoding="utf-8"))
    samples = [s for s in data["samples"] if not s["warmup"]]
    median = statistics.median(s["latency_ms"] for s in samples)
    rows.append({
        **record,
        "case": case,
        "samples": len(samples),
        "completed_twenty": sum(s["exactly_twenty_han"] and s["stop_reason"] == "sentence_boundary" for s in samples),
        "recomputed_p50_ms": median,
        "stored_p50_ms": data["p50_latency_ms_all_nonwarmup"],
        "p50_delta_ms": median - data["p50_latency_ms_all_nonwarmup"],
    })
result = {
    "config": {key: config[key] for key in ("name", "model", "model_reasoning_effort")},
    "config_sha256": hashlib.sha256(config_bytes).hexdigest(),
    "agents_original_is_prefix": prefix,
    "matrix_rows": rows,
}
print(json.dumps(result, ensure_ascii=False, indent=2))
print("END_UTC=" + datetime.now(timezone.utc).isoformat(), flush=True)
ok = (prefix and config["name"] == "routine_reporter" and config["model"] == "gpt-6-luna"
      and config["model_reasoning_effort"] == "max"
      and all(r["p50_delta_ms"] == 0 for r in rows) and len(rows) == 10)
raise SystemExit(0 if ok else 1)
