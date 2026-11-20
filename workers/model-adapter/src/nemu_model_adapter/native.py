"""In-process interface to the pinned b11669 Windows shim. No authority or IPC."""
from __future__ import annotations

import ctypes as ct
from contextlib import ExitStack
from dataclasses import dataclass
from hashlib import file_digest
from importlib.resources import files
import json
import os
from pathlib import Path
import sys

from .resources import Unmodulated


def sha256_file(path: Path) -> bytes:
    with path.open("rb") as f:
        return file_digest(f, "sha256").digest()


@dataclass(frozen=True)
class Assets:
    engine: Path
    model: Path
    tokenizer: Path
    mmproj: Path
    shim: Path
    shim_sha256: bytes  # trusted build/release digest supplied by the launcher


class Stats(ct.Structure):
    _fields_ = [("positions", ct.c_uint64), ("batches", ct.c_uint64),
                ("max_abs_error", ct.c_double), ("max_delta_l2", ct.c_double)]


@dataclass(frozen=True)
class Prompt:
    data: bytes
    literal_spans: tuple[int, ...]

    def cache_prefix(self) -> bytes:
        return (bytes([len(self.literal_spans)]) +
                b"".join(i.to_bytes(8, "little") for i in self.literal_spans) + self.data)


class Native:
    lib: ct.CDLL
    handle: int | None

    def __init__(self, assets: Assets, *, load_mmproj: bool = True) -> None:
        pins = json.loads(files(__package__).joinpath("manifest.json").read_bytes())
        self.model_hash = sha256_file(assets.model)
        self.tokenizer_hash = sha256_file(assets.tokenizer)
        self.model_bytes = assets.model.stat().st_size
        self.tokenizer_bytes = assets.tokenizer.stat().st_size
        if (self.model_hash.hex() != pins["gguf"]["carrier-Q4_K_M.gguf"]
                or self.tokenizer_hash.hex() != pins["tokenizer_sha256"]
                or sha256_file(assets.mmproj).hex() != pins["gguf"]["mmproj-F16.gguf"]
                or sha256_file(assets.shim) != assets.shim_sha256):
            raise Unmodulated("P0-12 model/tokenizer/mmproj or shim digest mismatch")
        engine = assets.engine.resolve(strict=True)
        expected = pins["engine_files"]
        if {p.name.lower() for p in engine.glob("*.dll")} != {k.lower() for k in expected}:
            raise Unmodulated("engine DLL inventory")
        for name, expected_hash in expected.items():
            if sha256_file(engine / name).hex() != expected_hash:
                raise Unmodulated(f"b11669 digest mismatch: {name}")
        self._search = ExitStack()
        if sys.platform == "win32":
            self._search.enter_context(os.add_dll_directory(str(engine)))
        else:
            raise Unmodulated("the pinned carrier requires Windows")
        try:
            # b11669's plugin loader calls LoadLibraryW. Load its dependencies from
            # the verified directory with ctypes' DLL_LOAD_DIR search before that call.
            self._backend_dlls = [ct.CDLL(str(engine / name)) for name in (
                "libomp.dll", "cudart64_12.dll", "cublasLt64_12.dll", "cublas64_12.dll", "ggml-cuda.dll")]
            self.lib = ct.CDLL(str(assets.shim.resolve(strict=True)))
            self._bind()
            handle = self.lib.carrier_open(str(engine).encode(), str(assets.model.resolve()).encode(),
                                           str(assets.mmproj.resolve()).encode() if load_mmproj else None)
            if not handle:
                self._fail()
            self.handle = handle
        except BaseException:
            self._search.close()
            raise

    def _bind(self) -> None:
        ptr, string, integer = ct.c_void_p, ct.c_char_p, ct.c_int
        signatures = {
            "carrier_error": (string, []),
            "carrier_open": (ptr, [string, string, string]),
            "carrier_close": (None, [ptr]),
            "carrier_begin": (ptr, [ptr, string, integer, ct.POINTER(integer), integer,
                                    integer, ct.POINTER(ct.c_double),
                                    integer, string, integer, ct.c_int64]),
            "carrier_next": (integer, [ptr, ct.POINTER(ptr), ct.POINTER(ct.c_size_t)]),
            "carrier_stats": (None, [ptr, ct.POINTER(Stats)]),
            "carrier_end": (None, [ptr]),
        }
        for name, (result, params) in signatures.items():
            fn = getattr(self.lib, name)
            fn.restype = result
            fn.argtypes = params

    def _fail(self) -> None:
        # The shim's errors contain subsystem names, never prompt/token text.
        raise Unmodulated(self.lib.carrier_error().decode("utf-8", errors="replace"))

    def begin(self, prompt: Prompt, layer: int, delta: tuple[float, ...], schema: bytes | None,
              max_tokens: int, duration_ns: int) -> int:
        if self.handle is None:
            raise Unmodulated("closed carrier")
        if not 0 < len(prompt.data) <= 2**31 - 1 or not 0 < duration_ns < 2**63:
            raise Unmodulated("native request bounds")
        if not 0 < max_tokens < 8192 or len(delta) != 2560:
            raise Unmodulated("P0-12 context or hook width")
        vector = (ct.c_double * len(delta))(*delta) if any(delta) else None
        spans = (ct.c_int * len(prompt.literal_spans))(*prompt.literal_spans)
        result: int = self.lib.carrier_begin(self.handle, prompt.data, len(prompt.data),
                                             spans, len(spans), layer, vector,
                                             len(delta), schema, max_tokens, duration_ns)
        if not result:
            self._fail()
        return result

    def next(self, request: int) -> bytes | None:
        piece, size = ct.c_void_p(), ct.c_size_t()
        rc = self.lib.carrier_next(request, ct.byref(piece), ct.byref(size))
        if rc < 0:
            self._fail()
        return ct.string_at(piece, size.value) if rc else None

    def stats(self, request: int) -> Stats:
        result = Stats()
        self.lib.carrier_stats(request, ct.byref(result))
        return result

    def end(self, request: int) -> None:
        self.lib.carrier_end(request)

    def close(self) -> None:
        if self.handle is not None:
            self.lib.carrier_close(self.handle)
            self.handle = None
            self._search.close()
