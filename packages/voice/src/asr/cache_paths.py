"""Local model/evaluation paths; no Windows system-drive cache fallback."""
from __future__ import annotations
import os
from pathlib import Path


def checked(value: Path) -> Path:
    if not value.is_absolute():
        raise ValueError('Nemu cache path must be absolute')
    if os.name == 'nt' and value.drive.lower() == 'c:':
        raise ValueError('Nemu model/evaluation caches must not use C:')
    return value


def cache_root() -> Path:
    default = Path('D:/NemuData') if os.name == 'nt' else Path(os.environ.get('XDG_CACHE_HOME', str(Path.home()/'.cache')))/'Nemu'
    return checked(Path(os.environ.get('NEMU_CACHE_ROOT', str(default))))


def data_directory() -> Path:
    return checked(Path(os.environ.get('NEMU_ASR_DATA', str(cache_root()/'asr-evaluation'))))


def evaluation_directory() -> Path:
    return cache_root()/'models/evaluation'


def configure_cache_environment() -> None:
    root = cache_root()
    os.environ.update(HF_HOME=str(root/'cache/huggingface'),
                      HF_HUB_CACHE=str(root/'cache/huggingface/hub'),
                      HF_DATASETS_CACHE=str(root/'cache/huggingface/datasets'),
                      HF_ASSETS_CACHE=str(root/'cache/huggingface/assets'),
                      TORCH_HOME=str(root/'cache/torch'),
                      XDG_CACHE_HOME=str(root/'cache'),
                      TEMP=str(root/'tmp'), TMP=str(root/'tmp'), TMPDIR=str(root/'tmp'))
