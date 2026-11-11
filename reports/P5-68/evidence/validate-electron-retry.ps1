# Exercise the exact workflow block with deterministic installer failures.
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path (Split-Path (Split-Path $PSScriptRoot -Parent) -Parent) -Parent
$lines = Get-Content (Join-Path $repoRoot '.github/workflows/ci.yml')
$inside = $false
$scriptLines = [System.Collections.Generic.List[string]]::new()
foreach ($line in $lines) {
    if ($line -match 'name: Install locked Electron binary before tests') { $inside = $true; continue }
    if (-not $inside) { continue }
    if ($line -eq '        run: |') { $collect = $true; continue }
    if ($collect) {
        if (-not $line.StartsWith('          ')) { break }
        $scriptLines.Add($line.Substring(10))
    }
}
if ($scriptLines.Count -eq 0) { throw 'Workflow block missing' }
$stub = @'
$script:attempts = 0
function node([string]$Path) {
    if ($Path -ne 'node_modules/electron/install.js') { throw 'Unexpected installer' }
    $script:attempts++
    Write-Output "INSTALL_ATTEMPT=$script:attempts"
    $global:LASTEXITCODE = if ($script:attempts -le FAILURE_COUNT) { 1 } else { 0 }
}
function Start-Sleep([int]$Seconds) {
    if ($Seconds -ne 15) { throw 'Unexpected retry delay' }
    Write-Output "RETRY_DELAY=$Seconds"
}
'@
$cache = 'D:/Dev/cache/P5-68/electron-retry-check'
New-Item -ItemType Directory -Path $cache -Force | Out-Null
foreach ($case in @(
    @{Name='first-success'; Failures=0; Exit=0; Attempts=1; Delays=0},
    @{Name='transient-failure'; Failures=2; Exit=0; Attempts=3; Delays=2},
    @{Name='persistent-failure'; Failures=3; Exit=1; Attempts=3; Delays=2}
)) {
    $path = Join-Path $cache ($case.Name + '.ps1')
    $stub.Replace('FAILURE_COUNT', [string]$case.Failures) + "`n" + ($scriptLines -join "`n") | Set-Content -LiteralPath $path
    $output = @(& pwsh -NoProfile -File $path)
    $code = $LASTEXITCODE
    if ($code -ne $case.Exit -or @($output -match '^INSTALL_ATTEMPT=').Count -ne $case.Attempts -or @($output -match '^RETRY_DELAY=').Count -ne $case.Delays) {
        throw "Failed $($case.Name): exit=$code; $output"
    }
    Write-Output "$($case.Name): exit=$code attempts=$($case.Attempts) delays=$($case.Delays) PASS"
}
