$ErrorActionPreference='Stop'
$env:UV_CACHE_DIR='D:\Dev\cache\uv'
$env:UV_PYTHON_INSTALL_DIR='D:\Dev\cache\uv-python'
$env:npm_config_cache='D:\Dev\cache\npm'
$env:PNPM_STORE_DIR='D:\Dev\cache\pnpm'
$env:CARGO_HOME='D:\Dev\cache\cargo'
$env:CARGO_TARGET_DIR='D:\Dev\cache\cargo-target\P5-04'
$env:ORT_CACHE_DIR='D:\Dev\cache\ort'
$env:HF_HOME='D:\Dev\cache\huggingface'
# Preserve the inherited temporary directory (Windows CI uses RUNNER_TEMP).
# Do not override TEMP/TMP or require ACL changes on a shared cache directory.
$env:NEMU_W19_MODEL='D:\Dev\models\multilingual-e5-small\614241f622f53c4eeff9890bdc4f31cfecc418b3'
$env:CUDA_VISIBLE_DEVICES=''
