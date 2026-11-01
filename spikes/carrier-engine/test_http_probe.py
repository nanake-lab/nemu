"""CPU-only checks of streaming measurements and cache/truncation rejection."""
from __future__ import annotations

import io
import json
from pathlib import Path
import tempfile
from typing import Any
import unittest
from unittest.mock import patch

from http_probe import Server


class StreamingTests(unittest.TestCase):
    def complete(self, output: Path, final: dict[str, Any]) -> dict[str, Any]:
        events = [{"content": "你", "tokens": [101], "stop": False},
                  {"content": "好。", "tokens": [102, 103], "stop": False}, final]
        raw = b"\n".join(b"data: " + json.dumps(event).encode() + b"\n" for event in events)
        server = Server(Path("unused"), Path("unused"), output.parent)
        server.endpoint = "http://127.0.0.1:1"
        with patch("http_probe.quiet"), patch("http_probe.urlopen", return_value=io.BytesIO(raw)):
            return server.completion({"prompt": [1, 2, 3]}, output)

    def test_final_empty_content_does_not_drop_stream_or_count_final_kv_as_reuse(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "request"
            result = self.complete(output, {"content": "", "tokens": [], "stop": True,
                "tokens_cached": 6, "truncated": False, "timings": {"prompt_n": 3, "cache_n": 0}})
            self.assertEqual(result["content"], "你好。")
            self.assertEqual(len(result["clientTokenTimesMs"]), 3)
            self.assertEqual(result["eventCount"], 3)
            self.assertLessEqual(result["ttftClientMs"], result["firstSentenceClientMs"])
            self.assertLessEqual(result["firstSentenceClientMs"], result["completedClientMs"])
            self.assertEqual(len((output / "events.jsonl").read_text(encoding="utf-8").splitlines()), 3)

    def test_reuse_truncation_and_incomplete_stream_retain_failures(self) -> None:
        cases = [{"stop": True, "timings": {"prompt_n": 2, "cache_n": 1}},
                 {"stop": True, "truncated": True, "timings": {"prompt_n": 3, "cache_n": 0}},
                 {"stop": False, "timings": {"prompt_n": 3, "cache_n": 0}}]
        for index, final in enumerate(cases):
            with self.subTest(index=index), tempfile.TemporaryDirectory() as directory:
                output = Path(directory) / "request"
                with self.assertRaises(ValueError):
                    self.complete(output, final)
                failure = json.loads((output / "failure.json").read_text(encoding="utf-8"))
                self.assertEqual(failure["errorType"], "ValueError")
                self.assertTrue((output / "request.json").is_file())
                self.assertTrue((output / "events.jsonl").is_file())

    def test_empty_completion_keeps_null_observed_ttft(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            server = Server(Path("unused"), Path("unused"), Path(directory))
            server.endpoint = "http://127.0.0.1:1"
            final = {"stop": True, "content": "", "tokens": [], "timings": {"prompt_n": 1, "cache_n": 0}}
            raw = b"data: " + json.dumps(final).encode() + b"\n"
            with patch("http_probe.quiet"), patch("http_probe.urlopen", return_value=io.BytesIO(raw)):
                result = server.completion({"prompt": [1]}, Path(directory) / "request")
            self.assertIsNone(result["ttftClientMs"])
            self.assertEqual(result["content"], "")
            self.assertEqual(result["clientTokenTimesMs"], [])


if __name__ == "__main__":
    unittest.main()
