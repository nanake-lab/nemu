"""Disable the optional SoX CLI capability before importing pinned Qwen 12 Hz.

qwen-tts 0.1.1 eagerly imports its unused 25 Hz encoder, which imports pysox.
pysox invokes `sox -h` during import. This service supports only locked 12 Hz
models: expose a fail-closed capability module instead of importing pysox.
No third-party source or global subprocess implementation is modified.
"""
from __future__ import annotations

import importlib
from importlib.machinery import ModuleSpec
import importlib.metadata
import sys
from threading import RLock
from types import ModuleType
from typing import NoReturn

_LOCK = RLock()
_MARKER = "__nemu_sox_disabled__"


def _deny(*args: object, **kwargs: object) -> NoReturn:
    raise RuntimeError("SOX_CLI_DISABLED_12HZ_ONLY")


def load_qwen() -> ModuleType:
    with _LOCK:
        if importlib.metadata.version("qwen-tts") != "0.1.1":
            raise RuntimeError("QWEN_API_VERSION_MISMATCH")
        existing = sys.modules.get("sox")
        if existing is not None and not getattr(existing, _MARKER, False):
            raise RuntimeError("SOX_ALREADY_IMPORTED_UNCONTROLLED")
        if existing is None:
            disabled = ModuleType("sox", "External SoX is disabled in Nemu's 12 Hz service")
            disabled.__spec__ = ModuleSpec("sox", loader=None)
            setattr(disabled, _MARKER, True)
            # The pinned Qwen code only references Transformer in its 25 Hz path.
            # Failing here prevents an unsupported capability from silently working.
            setattr(disabled, "Transformer", _deny)
            setattr(disabled, "Combiner", _deny)
            sys.modules["sox"] = disabled
        return importlib.import_module("qwen_tts")
