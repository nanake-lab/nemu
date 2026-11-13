import json
from pathlib import Path

from nemu_voice_sidecar.trace_writer import append_jsonl


def test_request_traces_append_to_one_jsonl_file(tmp_path: Path) -> None:
    target = tmp_path / "traces" / "requests.jsonl"
    for index in range(1200):
        append_jsonl(target, {"index": index})

    files = list(target.parent.iterdir())
    rows = [json.loads(line) for line in target.read_text(encoding="utf-8").splitlines()]
    assert files == [target]
    assert len(rows) == 1200
    assert rows[0] == {"index": 0}
    assert rows[-1] == {"index": 1199}
