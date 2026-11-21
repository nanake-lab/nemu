"""Unit tests for deterministic clause splitting and WebSocket deflate configuration."""
from __future__ import annotations

import ast
from pathlib import Path

from nemu_voice_sidecar.clauses import split_clauses


def test_single_clause_sentences_stay_unsplit() -> None:
    # Guards P4-02 deterministic splitting: single-clause sentences remain one fragment.
    assert split_clauses("今天的天气很好。") == ["今天的天气很好。"]
    assert split_clauses("The weather is nice today.") == ["The weather is nice today."]


def test_multi_clause_zh_and_en_split_deterministically() -> None:
    zh = "今天的天空很明亮，窗边有一点微风。我们可以慢慢说话，把眼前的小事一件一件做好。"
    en = "The sky is clear today, and a gentle breeze comes through the window. We can take our time, and finish each small task."
    expected_zh = [
        "今天的天空很明亮，",
        "窗边有一点微风。",
        "我们可以慢慢说话，",
        "把眼前的小事一件一件做好。",
    ]
    expected_en = [
        "The sky is clear today,",
        "and a gentle breeze comes through the window.",
        "We can take our time,",
        "and finish each small task.",
    ]
    assert split_clauses(zh) == expected_zh
    assert split_clauses(zh) == expected_zh  # reproducible across calls
    assert split_clauses(en) == expected_en
    assert split_clauses(en) == expected_en


def test_decimals_and_short_fragments_are_not_split_awkwardly() -> None:
    # Decimal points inside numbers must not split clauses.
    assert split_clauses("Version 3.14 is ready, please check the log.") == [
        "Version 3.14 is ready,",
        "please check the log.",
    ]
    # Short lead-ins (< 5 chars) merge into the following clause deterministically.
    assert split_clauses("你好，今天的天空很明亮。") == ["你好，今天的天空很明亮。"]
    assert split_clauses("晚安，祝你睡个好觉。") == ["晚安，祝你睡个好觉。"]


def test_punctuation_only_and_empty_inputs() -> None:
    assert split_clauses("") == []
    assert split_clauses("   ") == []
    assert split_clauses("……") == ["……"]


def test_main_disables_ws_per_message_deflate() -> None:
    # Guards P4-02 memory stability fix: loopback PCM transport must not enable zlib deflate.
    main_path = Path(__file__).resolve().parents[1] / "src" / "main.py"
    tree = ast.parse(main_path.read_text(encoding="utf-8"))
    found = False
    for node in ast.walk(tree):
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == "run":
            for kw in node.keywords:
                if kw.arg == "ws_per_message_deflate":
                    assert isinstance(kw.value, ast.Constant) and kw.value.value is False
                    found = True
    assert found, "uvicorn.run in src/main.py must explicitly set ws_per_message_deflate=False"
