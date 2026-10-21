"""CPU-only audit of the real pinned import chain; fail before any SoX spawn."""
from __future__ import annotations
import argparse
import hashlib
import importlib.metadata
import json
from pathlib import Path
import re
import sys
from gpu_common import add_service_packages, quiet_gate


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    quiet_gate()
    add_service_packages()
    attempts: list[str] = []
    def audit(event: str, values: tuple[object, ...]) -> None:
        if event not in ("subprocess.Popen", "os.system", "os.posix_spawn", "os.spawn"):
            return
        command = repr(values).lower()
        if re.search(r"\bsox(?:\.exe)?(?:[\s'\"]|$)", command):
            attempts.append(event)
            raise RuntimeError("SOX_PROCESS_ATTEMPT_DENIED")
    sys.addaudithook(audit)
    from nemu_voice_sidecar.qwen_import import load_qwen
    api = load_qwen()
    assert hasattr(api, "Qwen3TTSModel")
    disabled = sys.modules["sox"]
    assert getattr(disabled, "__nemu_sox_disabled__", False)
    denied = False
    try:
        getattr(disabled, "Transformer")()
    except RuntimeError as exc:
        denied = str(exc) == "SOX_CLI_DISABLED_12HZ_ONLY"
    assert denied and not attempts
    distribution = importlib.metadata.distribution("sox")
    original = Path(distribution.locate_file("sox/__init__.py"))
    result = {"qwen_version": importlib.metadata.version("qwen-tts"),
              "sox_process_attempts": attempts, "optional_sox_capability_denied": denied,
              "original_sox_init_sha256": hashlib.sha256(original.read_bytes()).hexdigest(),
              "cuda_initialized": sys.modules["torch"].cuda.is_initialized(),
              "method": "real Qwen import under a process-spawn audit; no weights loaded"}
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result), flush=True)


if __name__ == "__main__":
    main()
