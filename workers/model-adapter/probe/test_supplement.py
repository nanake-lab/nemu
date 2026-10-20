"""Evidence arithmetic and untrusted-output rendering checks; no GPU required."""
from __future__ import annotations

import json
from pathlib import Path
import subprocess
import sys

from summarize_supplement import summarize_memory


def test_memory_attribution_excludes_unowned_processes(tmp_path: Path) -> None:
    source = tmp_path / "memory.jsonl"
    row = dict(
        startedUtc="2026-10-20T00:00:00+00:00", endedUtc="2026-10-20T00:00:00.5+00:00",
        ownedProcesses=[dict(ProcessId=1, Name="python.exe"), dict(ProcessId=2, Name="chrome.exe")],
        counters=[dict(Name="pid_1_luid_a", DedicatedUsage=10, SharedUsage=2, TotalCommitted=12),
                  dict(Name="pid_2_luid_a", DedicatedUsage=20, SharedUsage=3, TotalCommitted=23),
                  dict(Name="pid_999_luid_a", DedicatedUsage=999, SharedUsage=0, TotalCommitted=999)],
    )
    source.write_text(json.dumps(row) + "\n", encoding="utf-8")
    result = summarize_memory(source)
    assert result["sampled_peaks"] == dict(dedicated_bytes=30, shared_bytes=5,
                                           total_committed_bytes=35)
    assert result["maximum_acquisition_s"] == 0.5
    assert result["status"] == "sampled-not-accepted"


def test_html_escapes_model_text_and_preserves_failed_stops(tmp_path: Path) -> None:
    source, output = tmp_path / "direction.json", tmp_path / "comparison.html"
    source.write_text(json.dumps(dict(
        model=dict(id="test/model"), generation_prompts=dict(zh="neutral"),
        samples=[dict(language="zh", seed=731, coefficient=-4,
                      output="<script>alert(1)</script>", stop="time_limit", elapsed_s=25.1)],
    )), encoding="utf-8")
    subprocess.run([sys.executable, str(Path(__file__).with_name("direction_html.py")),
                    str(source), "--output", str(output)], check=True)
    document = output.read_text(encoding="utf-8")
    assert "<script>" not in document
    assert "&lt;script&gt;alert(1)&lt;/script&gt;" in document
    assert "time_limit" in document and "-4" in document
