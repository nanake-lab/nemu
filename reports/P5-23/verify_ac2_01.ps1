param([Parameter(Mandatory = $true)][string]$EvidenceDirectory)

# Run the Rust recurrence itself; do not substitute a second implementation.
# The Rust test evaluates ADR 0018's inputs under ADR 0019. This acceptance command
# separately applies the frozen LP1 predicate and exits 1 on the violation.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$evidencePath = [IO.Path]::GetFullPath($EvidenceDirectory)
if (Test-Path -LiteralPath $evidencePath) { throw "Evidence directory already exists: $evidencePath" }
New-Item -ItemType Directory -Path $evidencePath | Out-Null
$rootPath = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$started = [DateTime]::UtcNow.ToString('o')
Push-Location (Join-Path $rootPath 'core')
try {
    & cargo test -p core-kernel --locked ac2_01_signal_limited_form_keeps_weak_support_monotone -- --nocapture 1> (Join-Path $evidencePath 'stdout.log') 2> (Join-Path $evidencePath 'stderr.log')
    $cargoExit = $LASTEXITCODE
} finally { Pop-Location }
@{ command = 'cargo test -p core-kernel --locked ac2_01_signal_limited_form_keeps_weak_support_monotone -- --nocapture'; started_utc = $started; ended_utc = [DateTime]::UtcNow.ToString('o'); exit_code = $cargoExit } |
    ConvertTo-Json | Set-Content -LiteralPath (Join-Path $evidencePath 'cargo-meta.json') -Encoding utf8
if ($cargoExit -ne 0) { exit $cargoExit }
$lines = @(Get-Content -LiteralPath (Join-Path $evidencePath 'stdout.log') | Where-Object { $_.StartsWith('AC2-01_RESULT=') })
if ($lines.Count -ne 1) { throw 'Expected exactly one AC2-01 result from Rust' }
$result = $lines[0].Substring('AC2-01_RESULT='.Length) | ConvertFrom-Json
$result | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $evidencePath 'result.json') -Encoding utf8
$result | ConvertTo-Json | Write-Output
if ($result.status -eq 'failed') { exit 1 }
if ($result.status -ne 'passed') { throw 'Unknown acceptance status' }
exit 0
