"""Generate test-only synthetic references; never consumes any human recording."""
from __future__ import annotations

import argparse
import gc
import hashlib
import importlib
import json
from pathlib import Path

from gpu_common import ROOT, add_service_packages, quiet_gate

TEXTS = {
    "zh": "今天的天空很明亮，窗边有一点微风。我们可以慢慢说话，把眼前的小事一件一件做好。",
    "en": "The sky is clear today, and a gentle breeze comes through the window. We can take our time and finish the little things one by one.",
    "ja": "今日は空がとても明るくて、窓から優しい風が入ってきます。ゆっくり話しながら、目の前の小さなことを一つずつ進めましょう。",
}
DESCRIPTION = ("An original fictional adult female voice, gentle and clear, medium pitch, "
               "natural calm delivery. Do not imitate any real person or existing character.")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    quiet_gate()
    add_service_packages()
    from nemu_voice_sidecar.models import verified_model
    metadata = json.loads((ROOT / "config/models.lock.json").read_text(encoding="utf-8-sig"))["tts"]["models"]["VoiceDesign"]
    torch = importlib.import_module("torch")
    sf = importlib.import_module("soundfile")
    api = importlib.import_module("qwen_tts")
    model = api.Qwen3TTSModel.from_pretrained(str(verified_model(args.cache, "VoiceDesign")),
                                        local_files_only=True, device_map="cuda:0",
                                        dtype=torch.bfloat16, attn_implementation="eager")
    args.output.mkdir(parents=True, exist_ok=False)
    entries = {}
    manifest = []
    for lang, text in TEXTS.items():
        quiet_gate()
        torch.manual_seed(731)
        wavs, rate = model.generate_voice_design(text=text, language={"zh": "Chinese", "en": "English", "ja": "Japanese"}[lang],
                                               instruct=DESCRIPTION, max_new_tokens=384)
        path = args.output / f"{lang}.wav"
        sf.write(path, wavs[0], rate, subtype="PCM_16")
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        ref = {"path": path.name, "text": text, "sha256": digest, "provenance": "VoiceDesign",
               "source": f"{metadata['source']}/tree/{metadata['revision']}",
               "license": "Apache-2.0", "cloneAllowed": True}
        entries[lang] = {"neutral": ref}
        manifest.append({"path": path.name, "kind": "voice", "source": ref["source"],
                         "license": "Apache-2.0", "redistributable": True,
                         "sha256": digest, "seconds": len(wavs[0]) / rate,
                         "notes": "Test-only VoiceDesign output, no human reference; not Nemu's accepted voice"})
        print(f"Generated synthetic {lang}, {len(wavs[0]) / rate:.2f}s", flush=True)
    (args.output / "voice.json").write_text(json.dumps({"id": "p4-02-synthetic-test", "references": entries,
                                                    "description": DESCRIPTION}, ensure_ascii=False, indent=2), encoding="utf-8")
    (args.output / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    del model
    gc.collect()
    torch.cuda.empty_cache()


if __name__ == "__main__":
    main()
