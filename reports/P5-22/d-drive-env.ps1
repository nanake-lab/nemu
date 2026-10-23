# Dot-source before checks; keep all newly created caches and scratch files on D:.
$cacheRoot = 'D:\Dev\cache\P5-22'
foreach ($name in @('tmp','npm','pnpm','corepack','node-compile','uv','uv-python','pycache','mypy','ruff','xdg','localappdata','dotnet','nuget')) {
  New-Item -ItemType Directory -Force -Path (Join-Path $cacheRoot $name) | Out-Null
}
$env:TEMP = Join-Path $cacheRoot 'tmp'
$env:TMP = $env:TEMP
$env:npm_config_cache = Join-Path $cacheRoot 'npm'
$env:npm_config_store_dir = 'D:\.pnpm-store'
$env:PNPM_HOME = Join-Path $cacheRoot 'pnpm'
$env:COREPACK_HOME = Join-Path $cacheRoot 'corepack'
$env:COREPACK_ENABLE_DOWNLOAD_PROMPT = '0'
$env:NODE_COMPILE_CACHE = Join-Path $cacheRoot 'node-compile'
$env:UV_CACHE_DIR = Join-Path $cacheRoot 'uv'
$env:UV_PYTHON_INSTALL_DIR = 'D:\Dev\cache\uv-python'
$env:PYTHONPYCACHEPREFIX = Join-Path $cacheRoot 'pycache'
$env:MYPY_CACHE_DIR = Join-Path $cacheRoot 'mypy'
$env:RUFF_CACHE_DIR = Join-Path $cacheRoot 'ruff'
$env:XDG_CACHE_HOME = Join-Path $cacheRoot 'xdg'
$env:LOCALAPPDATA = Join-Path $cacheRoot 'localappdata'
$env:CARGO_HOME = 'D:\Dev\cache\cargo'
$env:DOTNET_CLI_HOME = Join-Path $cacheRoot 'dotnet'
$env:NUGET_PACKAGES = Join-Path $cacheRoot 'nuget'
$env:PYTHONUTF8 = '1'
$env:UV_LINK_MODE = 'copy'
