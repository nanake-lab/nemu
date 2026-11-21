from __future__ import annotations

import hashlib
import json
from pathlib import Path

import pytest

from nemu_voice_sidecar.models import verified_model


def test_cached_model_is_hash_verified(tmp_path: Path) -> None:
    folder = tmp_path / 'Qwen--test' / ('a' * 40)
    folder.mkdir(parents=True)
    model = folder / 'model.safetensors'
    model.write_bytes(b'test fixture')
    lock = tmp_path / 'models.lock.json'
    lock.write_text(json.dumps({'tts': {'models': {'test': {
        'id': 'Qwen/test', 'revision': 'a' * 40, 'files': [
            {'path': model.name, 'size': model.stat().st_size,
             'sha256': hashlib.sha256(model.read_bytes()).hexdigest()}]}}}}))
    assert verified_model(tmp_path, 'test', lock) == folder
    model.write_bytes(b'bad! fixture')
    with pytest.raises(ValueError, match='HASH'):
        verified_model(tmp_path, 'test', lock)
    model.unlink()
    with pytest.raises(ValueError, match='MISSING'):
        verified_model(tmp_path, 'test', lock)


def test_model_registry_schema_and_license() -> None:
    from jsonschema import Draft202012Validator
    root = Path(__file__).resolve().parents[3]
    lock = json.loads((root / 'config/models.lock.json').read_text(encoding='utf-8-sig'))
    schema = json.loads((root / 'config/models.lock.schema.json').read_text(encoding='utf-8-sig'))
    Draft202012Validator(schema).validate(lock)
    assert set(lock['tts']['models']) == {'0.6B', '1.7B', 'VoiceDesign'}
    for model in lock['tts']['models'].values():
        assert model['license'] == 'Apache-2.0'
        assert len(model['revision']) == 40
        assert len(model['files']) == 11
