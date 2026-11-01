param([ValidateSet('install','test')][string]$Step, [int]$Attempt = 1)
$ErrorActionPreference = 'Stop'
$taskRoot = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$baselineRoot = 'D:\Dev\nemu-wt\P5-34-main-check'
$cacheRoot = 'D:\Dev\cache\P5-34'
$env:COREPACK_HOME = Join-Path $cacheRoot 'corepack'
$env:NODE_COMPILE_CACHE = Join-Path $cacheRoot 'node-compile'
$env:TEMP = Join-Path $cacheRoot 'tmp'
$env:TMP = $env:TEMP
$busy = @(Get-CimInstance Win32_Process | Where-Object { $_.Name -match '^(Runner\.Worker\.exe|cargo\.exe|rustc\.exe|core_store.*\.exe)$' })
if ($busy.Count -ne 0) {
    $busy | Select-Object Name,ProcessId,ParentProcessId,CreationDate,CommandLine | ConvertTo-Json -Depth 4 | Write-Output
    throw 'Resource window busy; baseline command not started.'
}
Set-Location -LiteralPath $baselineRoot
$head = (& git rev-parse HEAD).Trim()
if ($head -ne 'b4eb7fd52f848eb042b2446c3f4ebf2416108c6c') { throw 'Unexpected baseline revision' }
$folder = Join-Path $PSScriptRoot 'verification/protected-20261026'
New-Item -ItemType Directory -Force -Path $folder | Out-Null
$stem = 'main-{0}-{1:d2}' -f $Step,$Attempt
$log = Join-Path $folder ($stem + '.log')
$meta = Join-Path $folder ($stem + '.json')
if ((Test-Path -LiteralPath $log) -or (Test-Path -LiteralPath $meta)) { throw 'Evidence already exists' }
$nativeArgs = @(if ($Step -eq 'install') { @('install','--frozen-lockfile','--store-dir','D:\Dev\cache\pnpm-store') } else { @('test') })
$record = [ordered]@{ command=@('pnpm') + $nativeArgs; cwd=$baselineRoot; head=$head; node=(& node --version); pnpm=(& pnpm --version); startUtc=(Get-Date).ToUniversalTime().ToString('o'); resourceGuard='no Runner.Worker/cargo/rustc/core_store before start'; comparison='Predeclared 3 full pnpm test runs on unchanged main. Attempts 01-03 exposed scalar argument splatting (vitest run e s t); retained as harness-error evidence. Attempts 04-06 are the corrected fixed-size comparison, not run-until-pass.' }
New-Item -ItemType File -Path $log | Out-Null
$ErrorActionPreference = 'Continue'
& pnpm @nativeArgs 2>&1 | Tee-Object -FilePath $log
$code = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
$record.endUtc = (Get-Date).ToUniversalTime().ToString('o')
$record.exitCode = $code
$record.logSha256 = (Get-FileHash -LiteralPath $log -Algorithm SHA256).Hash.ToLowerInvariant()
$record | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $meta -Encoding utf8
Write-Output ('BASELINE {0} ATTEMPT {1} EXIT {2}' -f $Step,$Attempt,$code)
exit $code
