from __future__ import annotations
import sys
from types import ModuleType
import pytest
from nemu_voice_sidecar.qwen_import import load_qwen


def test_optional_sox_is_fail_closed(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delitem(sys.modules, "sox", raising=False)
    monkeypatch.setattr("nemu_voice_sidecar.qwen_import.importlib.metadata.version", lambda _: "0.1.1")
    fake = ModuleType("qwen_tts")
    def imported(name: str) -> ModuleType:
        assert name == "qwen_tts"
        disabled = sys.modules["sox"]
        assert getattr(disabled, "__nemu_sox_disabled__")
        with pytest.raises(RuntimeError, match="SOX_CLI_DISABLED"):
            getattr(disabled, "Transformer")()
        return fake
    monkeypatch.setattr("nemu_voice_sidecar.qwen_import.importlib.import_module", imported)
    assert load_qwen() is fake
    monkeypatch.delitem(sys.modules, "sox")


def test_uncontrolled_sox_import_is_rejected(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr("nemu_voice_sidecar.qwen_import.importlib.metadata.version", lambda _: "0.1.1")
    monkeypatch.setitem(sys.modules, "sox", ModuleType("sox"))
    with pytest.raises(RuntimeError, match="ALREADY_IMPORTED"):
        load_qwen()
