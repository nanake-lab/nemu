param([Parameter(Mandatory)][string]$Step, [int]$Attempt = 1)
$ErrorActionPreference = 'Stop'
$taskRoot = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$taskCache = 'D:\Dev\cache\P5-34'
$env:CARGO_HOME = 'D:\Dev\cache\cargo'
$env:CARGO_TARGET_DIR = Join-Path $taskCache 'target'
$env:CARGO_BUILD_JOBS = '1'
$env:RUST_TEST_THREADS = '1'
$env:UV_CACHE_DIR = 'D:\Dev\cache\uv'
$env:UV_PYTHON_INSTALL_DIR = 'D:\Dev\cache\uv-python'
$env:UV_PROJECT_ENVIRONMENT = Join-Path $taskCache 'contracts-venv'
$env:PYTHONPYCACHEPREFIX = Join-Path $taskCache 'pycache'
$env:MYPY_CACHE_DIR = Join-Path $taskCache 'mypy'
$env:RUFF_CACHE_DIR = Join-Path $taskCache 'ruff'
$env:NODE_COMPILE_CACHE = Join-Path $taskCache 'node-compile'
$env:COREPACK_HOME = Join-Path $taskCache 'corepack'
$env:TEMP = Join-Path $taskCache 'tmp'
$env:TMP = $env:TEMP
$env:PYTHONUTF8 = '1'
Set-Location -LiteralPath $taskRoot
$commands = @{
    generate = @('python', 'core/contracts/generate.py', '--check')
    storage = @('cargo', '+1.99.0', 'test', '--manifest-path', 'core/Cargo.toml', '-p', 'core-store', '--locked', '-j', '1', 'p534_', '--', '--test-threads=1')
    types = @('cargo', '+1.99.0', 'test', '--manifest-path', 'core/Cargo.toml', '-p', 'core-types', '--locked', '-j', '1', '--', '--test-threads=1')
    clippy = @('cargo', '+1.99.0', 'clippy', '--manifest-path', 'core/Cargo.toml', '--workspace', '--all-targets', '--locked', '-j', '1', '--', '-D', 'warnings')
    workspace = @('cargo', '+1.99.0', 'test', '--manifest-path', 'core/Cargo.toml', '--workspace', '--locked', '-j', '1', '--', '--test-threads=1')
    remaining = @('cargo', '+1.99.0', 'test', '--manifest-path', 'core/Cargo.toml', '-p', 'echoctl', '-p', 'fact-view', '-p', 'gateway', '-p', 'semantic-core', '-p', 'task-layer', '--locked', '-j', '1', '--', '--test-threads=1')
    doctests = @('cargo', '+1.99.0', 'test', '--manifest-path', 'core/Cargo.toml', '--workspace', '--doc', '--locked', '-j', '1', '--', '--test-threads=1')
    contracts = @('pnpm', 'test:core-contracts')
    compatibility = @('python', 'core/contracts/check_compat.py', '--ref', 'origin/main')
    bytes = @('python', 'reports/P5-21/review_vectors.py', '--base', 'origin/main')
    inputs = @('python', 'reports/P5-34/check_retention_inputs.py')
    fmt = @('cargo', '+1.99.0', 'fmt', '--manifest-path', 'core/Cargo.toml', '--all', '--', '--check')
    lint = @('pnpm', 'lint')
    typecheck = @('pnpm', 'typecheck')
    test = @('pnpm', 'test')
    build = @('pnpm', 'build')
    ruff = @('uv', 'run', '--project', 'core/contracts', '--locked', 'ruff', 'check', 'core/contracts', 'workers/irl/generated/echo', 'workers/drive/generated/echo', 'workers/model-adapter/generated/echo')
    mypy = @('uv', 'run', '--project', 'core/contracts', '--locked', 'mypy', '--strict', 'core/contracts/generate.py', 'core/contracts/fixture_inputs.py', 'core/contracts/fixture_retention.py', 'core/contracts/adversarial.py', 'core/contracts/verify.py', 'core/contracts/check_compat.py', 'core/contracts/freeze_vectors.py', 'workers/irl/generated/echo')
}
foreach ($worker in @('irl', 'drive', 'model-adapter')) {
    $commands["$worker-pytest"] = @('uv', 'run', '--project', "workers/$worker", '--locked', 'pytest', "workers/$worker/tests")
    $commands["$worker-mypy"] = @('uv', 'run', '--project', "workers/$worker", '--locked', 'mypy', '--strict', "workers/$worker/src", "workers/$worker/tests")
    $commands["$worker-ruff"] = @('uv', 'run', '--project', "workers/$worker", '--locked', 'ruff', 'check', "workers/$worker/src", "workers/$worker/tests", "workers/$worker/generated")
    if ($Step.StartsWith("$worker-")) {
        $env:UV_PROJECT_ENVIRONMENT = Join-Path $taskCache "venvs/$worker"
        $env:PYTHONPATH = Join-Path $taskRoot "workers/$worker/src"
    }
}
if (-not $commands.ContainsKey($Step)) { throw 'Unknown step.' }
$light = $Step -in @('generate', 'compatibility', 'bytes', 'inputs', 'ruff', 'mypy', 'fmt', 'lint') -or $Step.EndsWith('-ruff') -or $Step.EndsWith('-mypy')
if (-not $light) {
    $busy = @(Get-CimInstance Win32_Process | Where-Object { $_.Name -match '^(Runner\.Worker\.exe|cargo\.exe|rustc\.exe|core_store.*\.exe)$' })
    if ($busy.Count -ne 0) {
        $guardFolder = Join-Path $PSScriptRoot 'verification/protected-20261026'
        New-Item -ItemType Directory -Force -Path $guardFolder | Out-Null
        $guardPath = Join-Path $guardFolder ('guard-{0}-{1}.json' -f $Step, (Get-Date).ToUniversalTime().ToString('yyyyMMddTHHmmssfff'))
        [ordered]@{ step=$Step; commandsStarted=$false; observedUtc=(Get-Date).ToUniversalTime().ToString('o'); processes=@($busy | Select-Object Name,ProcessId,ParentProcessId,CreationDate,CommandLine) } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $guardPath -Encoding utf8
        throw 'Resource window busy; no verification command started.'
    }
}
$folder = Join-Path $PSScriptRoot ('verification/protected-20261026/attempt-{0:d2}' -f $Attempt)
New-Item -ItemType Directory -Force -Path $folder | Out-Null
$log = Join-Path $folder ($Step + '.log')
$meta = Join-Path $folder ($Step + '.json')
if ((Test-Path -LiteralPath $log) -or (Test-Path -LiteralPath $meta)) { throw 'Evidence already exists.' }
$argv = $commands[$Step]
$metadata = [ordered]@{ step=$Step; command=$argv; cwd=$taskRoot; start=(Get-Date).ToUniversalTime().ToString('o'); head=(& git rev-parse HEAD).Trim(); main=(& git rev-parse origin/main).Trim(); target=$env:CARGO_TARGET_DIR; temp=$env:TEMP; jobs=1; testThreads=1 }
New-Item -ItemType File -Path $log | Out-Null
$exe = $argv[0]
$nativeArgs = $argv[1..($argv.Length-1)]
$ErrorActionPreference = 'Continue'
& $exe @nativeArgs 2>&1 | Tee-Object -FilePath $log
$code = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
$metadata.end = (Get-Date).ToUniversalTime().ToString('o')
$metadata.exitCode = $code
$metadata.logSha256 = (Get-FileHash -LiteralPath $log -Algorithm SHA256).Hash.ToLowerInvariant()
$metadata | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $meta -Encoding utf8
Write-Output ('STEP {0} EXIT {1}' -f $Step, $code)
exit $code
