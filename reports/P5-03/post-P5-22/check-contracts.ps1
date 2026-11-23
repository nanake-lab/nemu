$ErrorActionPreference = 'Stop'
$env:CARGO_HOME = 'D:\Dev\cache\cargo'
$env:CARGO_TARGET_DIR = 'D:\Dev\cache\cargo-target\P5-03'
$env:UV_CACHE_DIR = 'D:\Dev\cache\uv'
$env:UV_PYTHON_INSTALL_DIR = 'D:\Dev\cache\uv-python'
$env:UV_PROJECT_ENVIRONMENT = 'D:\Dev\cache\P5-03\contracts-venv'
$env:npm_config_cache = 'D:\Dev\cache\npm'
$env:PNPM_STORE_DIR = 'D:\Dev\cache\pnpm'
$env:npm_config_cache_dir = 'D:\Dev\cache\pnpm-metadata'
$env:PNPM_HOME = 'D:\Dev\cache\P5-03\pnpm-home'
$env:COREPACK_HOME = 'D:\Dev\cache\node\corepack'
$env:COREPACK_ENABLE_DOWNLOAD_PROMPT = '0'
$env:XDG_CACHE_HOME = 'D:\Dev\cache'
$env:XDG_STATE_HOME = 'D:\Dev\cache\state'
$env:NODE_COMPILE_CACHE = 'D:\Dev\cache\node-compile\P5-03'
$env:ELECTRON_CACHE = 'D:\Dev\cache\electron'
$env:PLAYWRIGHT_BROWSERS_PATH = 'D:\Dev\cache\playwright'
$env:HF_HOME = 'D:\Dev\cache\huggingface'
$env:TEMP = 'D:\Dev\cache\P5-03\tmp'
$env:TMP = $env:TEMP
$env:PYTHONDONTWRITEBYTECODE = '1'
$env:PYTHONUTF8 = '1'
$repo = Split-Path (Split-Path (Split-Path $PSScriptRoot -Parent) -Parent) -Parent
Push-Location $repo
try {
    pnpm install --frozen-lockfile --store-dir $env:PNPM_STORE_DIR --state-dir D:\Dev\cache\pnpm-state
    if ($LASTEXITCODE -ne 0) { throw 'pnpm install failed' }
    uv sync --project core/contracts --locked
    if ($LASTEXITCODE -ne 0) { throw 'contract environment sync failed' }
    # verify.py's frozen probe path is core/target. Point it at the same D-drive
    # target directory cargo uses; do not edit the contract tool out of scope.
    $probeTarget = Join-Path $repo 'core/target'
    if (-not (Test-Path -LiteralPath $probeTarget)) {
        New-Item -ItemType Junction -Path $probeTarget -Target $env:CARGO_TARGET_DIR | Out-Null
    }
    $actual = (Get-Item -LiteralPath $probeTarget).Target
    if ($actual -ne $env:CARGO_TARGET_DIR) { throw 'unexpected contract probe target' }
    python core/contracts/check_compat.py --ref origin/main
    if ($LASTEXITCODE -ne 0) { throw 'origin/main compatibility failed' }
    pnpm test:contracts
    if ($LASTEXITCODE -ne 0) { throw 'terminal contracts failed' }
    pnpm test:core-contracts
    if ($LASTEXITCODE -ne 0) { throw 'three-language contract verification failed' }
} finally { Pop-Location }
