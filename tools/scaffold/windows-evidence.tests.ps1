$ErrorActionPreference = 'Stop'
$script = Join-Path $PSScriptRoot 'windows-evidence.ps1'
$env:RUNNER_TEMP = Join-Path ([IO.Path]::GetTempPath()) ('nemu-evidence-tests-' + [guid]::NewGuid())
$env:GITHUB_RUN_ID = 'fixture'
$env:GITHUB_RUN_ATTEMPT = '1'
$env:GITHUB_ENV = ''
function Invoke-Case([string]$Mode, [bool]$Success) {
    & pwsh -NoProfile -File $script -Mode $Mode
    if (($LASTEXITCODE -eq 0) -ne $Success) { throw "Unexpected exit for $Mode (expected success=$Success)" }
}
Invoke-Case Init $true
$env:NEMU_EVIDENCE_DIR = Join-Path $env:RUNNER_TEMP 'nemu-evidence/fixture/1/windows'
Invoke-Case Init $false
$file = Join-Path $env:NEMU_EVIDENCE_DIR 'fixture.txt'
Set-Content -LiteralPath $file -Value 'fresh'
Invoke-Case Verify $true
(Get-Item -LiteralPath $file).LastWriteTimeUtc = [DateTime]::UtcNow.AddDays(-1)
Invoke-Case Verify $false
(Get-Item -LiteralPath $file).LastWriteTimeUtc = [DateTime]::UtcNow
$env:GITHUB_RUN_ATTEMPT = '2'
Invoke-Case Verify $false
$env:GITHUB_RUN_ATTEMPT = '1'
Invoke-Case Verify $true
Write-Output 'PASS: fresh evidence accepted; duplicate directory, stale timestamp and wrong attempt rejected.'
