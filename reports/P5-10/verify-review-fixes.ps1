param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('fmt', 'kernel', 'clippy', 'workspace')]
    [string]$Step
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$evidenceRoot = Join-Path $PSScriptRoot 'review-fixes-2026-10-26'
$taskCache = 'D:\Dev\cache\P5-10'
$tempRoot = Join-Path $taskCache 'tmp'
New-Item -ItemType Directory -Force -Path $evidenceRoot, $tempRoot | Out-Null
$env:CARGO_HOME = 'D:\Dev\cache\cargo'
$env:CARGO_TARGET_DIR = Join-Path $taskCache 'target'
$env:TEMP = $tempRoot
$env:TMP = $tempRoot
$env:CARGO_BUILD_JOBS = '2'
$env:RUST_TEST_THREADS = '1'

$busy = @(Get-Process -Name cargo, rustc, Runner.Worker -ErrorAction SilentlyContinue)
if ($busy.Count -gt 0) {
    throw ('Heavy workload already running: ' + (($busy | ForEach-Object { "$($_.Name):$($_.Id)" }) -join ', '))
}
$cargoArgs = switch ($Step) {
    'fmt' { @('fmt', '--all', '--', '--check') }
    'kernel' { @('test', '-p', 'core-kernel', '--locked', '--offline') }
    'clippy' { @('clippy', '--workspace', '--all-targets', '--locked', '--offline', '--', '-D', 'warnings') }
    'workspace' { @('test', '--workspace', '--locked', '--offline') }
}
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$logPath = Join-Path $evidenceRoot "$Step-$stamp.log"
$start = Get-Date
Push-Location (Join-Path $repoRoot 'core')
try {
    @(
        "Started: $($start.ToString('o'))"
        "Step: $Step"
        "Command: cargo $($cargoArgs -join ' ')"
        "Working directory: $((Get-Location).Path)"
        "HEAD before review-fix commit: $(git rev-parse HEAD)"
        "Origin main: $(git rev-parse origin/main)"
        "Toolchain: $(rustc --version)"
        "CARGO_HOME=$env:CARGO_HOME"
        "CARGO_TARGET_DIR=$env:CARGO_TARGET_DIR"
        "TEMP=$env:TEMP"
        "CARGO_BUILD_JOBS=$env:CARGO_BUILD_JOBS RUST_TEST_THREADS=$env:RUST_TEST_THREADS"
        'Data: synthetic fixtures and fresh temporary stores only'
    ) | Tee-Object -FilePath $logPath
    & cargo @cargoArgs 2>&1 | Tee-Object -FilePath $logPath -Append
    $code = $LASTEXITCODE
    $end = Get-Date
    @("Finished: $($end.ToString('o'))", "Elapsed seconds: $(($end - $start).TotalSeconds)", "Exit code: $code") |
        Tee-Object -FilePath $logPath -Append
} finally {
    Pop-Location
}
exit $code
