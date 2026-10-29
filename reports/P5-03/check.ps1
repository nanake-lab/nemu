$ErrorActionPreference = 'Stop'
$env:CARGO_HOME = 'D:\Dev\cache\cargo'
$env:CARGO_TARGET_DIR = 'D:\Dev\cache\cargo-target\P5-03'
$env:UV_CACHE_DIR = 'D:\Dev\cache\uv'
$env:UV_PYTHON_INSTALL_DIR = 'D:\Dev\cache\uv-python'
$env:npm_config_cache = 'D:\Dev\cache\npm'
$env:PNPM_STORE_DIR = 'D:\Dev\cache\pnpm'
$env:HF_HOME = 'D:\Dev\cache\huggingface'
$env:TORCH_HOME = 'D:\Dev\cache\torch'
$env:TORCHINDUCTOR_CACHE_DIR = 'D:\Dev\cache\torchinductor'
$env:TEMP = 'D:\Dev\cache\P5-03\tmp'
$env:TMP = $env:TEMP
$env:PYTHONDONTWRITEBYTECODE = '1'
New-Item -ItemType Directory -Force $env:TEMP | Out-Null
$repo = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
Push-Location (Join-Path $repo 'core')
try {
    cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'cargo fmt failed' }
    cargo clippy --workspace --all-targets --locked -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'cargo clippy failed' }
    cargo test --workspace --locked -- --test-threads=1
    if ($LASTEXITCODE -ne 0) { throw 'cargo test failed' }
} finally { Pop-Location }
