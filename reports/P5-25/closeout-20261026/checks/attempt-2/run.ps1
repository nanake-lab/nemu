$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../../../../..')).Path
$env:CARGO_HOME = 'D:/Dev/cache/cargo'
$env:CARGO_TARGET_DIR = 'D:/Dev/cache/P5-25/target'
$env:CARGO_BUILD_JOBS = '1'
$env:TEMP = 'D:/Dev/cache/P5-25/tmp'
$env:TMP = $env:TEMP
New-Item -ItemType Directory -Force -Path $env:TEMP | Out-Null
Set-Location (Join-Path $repo 'core')
if (Get-Process -Name cargo,rustc,Runner.Worker -ErrorAction SilentlyContinue) {
    throw 'Another heavy workload is active; no checks started.'
}
rustc --version | Set-Content (Join-Path $PSScriptRoot 'toolchain.txt')
cargo --version | Add-Content (Join-Path $PSScriptRoot 'toolchain.txt')
git rev-parse HEAD | Add-Content (Join-Path $PSScriptRoot 'toolchain.txt')
git diff -- crates/core-kernel/Cargo.toml Cargo.lock |
    Set-Content (Join-Path $PSScriptRoot 'dependency.diff')
$checks = @(
    @{ Name = 'fmt'; Args = @('fmt', '--all', '--', '--check'); Expected = 0 },
    @{ Name = 'clippy'; Args = @('clippy', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings'); Expected = 0 },
    @{ Name = 'workspace'; Args = @('test', '--workspace', '--locked', '--no-fail-fast'); Expected = 0 },
    @{ Name = 'legacy-diagnostic'; Args = @('test', '-p', 'core-kernel', '--locked', 'dynamics::tests::reference::theta_1_original_500_streams_zero_violations', '--', '--exact', '--ignored', '--nocapture'); Expected = 101 }
)
$results = @()
foreach ($check in $checks) {
    $started = [DateTime]::UtcNow
    $arguments = $check.Args
    & cargo @arguments *> (Join-Path $PSScriptRoot ($check.Name + '.log'))
    $code = $LASTEXITCODE
    $results += [ordered]@{
        command = 'cargo ' + ($arguments -join ' ')
        exit_code = $code
        expected_exit_code = $check.Expected
        result = $(if ($code -eq 0) { 'passed' } else { 'failed' })
        expected_outcome_observed = ($code -eq $check.Expected)
        started_utc = $started.ToString('o')
        ended_utc = [DateTime]::UtcNow.ToString('o')
    }
    ConvertTo-Json -InputObject @($results) -Depth 5 | Set-Content (Join-Path $PSScriptRoot 'results.json')
    Write-Output ($check.Name + ': exit ' + $code)
    if ($code -ne $check.Expected) { exit 1 }
}
