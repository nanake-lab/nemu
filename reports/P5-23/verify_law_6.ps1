param([Parameter(Mandatory = $true)][string]$EvidenceDirectory)

# Original LAW-6 predicate/seed/3000 sequences, run on Rust ADR 0019.
# The diagnostic regression test passing does not mean LAW-6 passes.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$evidencePath = [IO.Path]::GetFullPath($EvidenceDirectory)
if (Test-Path -LiteralPath $evidencePath) { throw "Evidence already exists: $evidencePath" }
New-Item -ItemType Directory -Path $evidencePath | Out-Null
$rootPath = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$started = [DateTime]::UtcNow.ToString('o')
Push-Location (Join-Path $rootPath 'core')
try {
    & cargo test -p core-kernel --locked law_6_original_predicate_failure_is_retained -- --nocapture 1> (Join-Path $evidencePath 'stdout.log') 2> (Join-Path $evidencePath 'stderr.log')
    $cargoExit = $LASTEXITCODE
} finally { Pop-Location }
@{command='cargo test -p core-kernel --locked law_6_original_predicate_failure_is_retained -- --nocapture';started_utc=$started;ended_utc=[DateTime]::UtcNow.ToString('o');exit_code=$cargoExit} |
    ConvertTo-Json | Set-Content (Join-Path $evidencePath 'cargo-meta.json') -Encoding utf8
if ($cargoExit -ne 0) { exit $cargoExit }
$lines = @(Get-Content (Join-Path $evidencePath 'stdout.log') | Where-Object { $_.StartsWith('LAW-6_RESULT=') })
if ($lines.Count -ne 1) { throw 'Expected one LAW-6 result from Rust' }
$result = $lines[0].Substring('LAW-6_RESULT='.Length) | ConvertFrom-Json
$result | ConvertTo-Json | Set-Content (Join-Path $evidencePath 'result.json') -Encoding utf8
$result | ConvertTo-Json | Write-Output
if ($result.status -eq 'failed') { exit 1 }
if ($result.status -ne 'passed') { throw 'Unknown acceptance status' }
exit 0
