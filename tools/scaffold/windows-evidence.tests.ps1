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

. (Join-Path $PSScriptRoot 'windows-snapshot.ps1')
# Synthetic names/paths: never serialize a real background process list in a test.
$repository = Join-Path $env:RUNNER_TEMP 'checkout'
function Get-Process {
    param($Id)
    if ($Id) { return [pscustomobject]@{ SessionId = 1 } }
    $rows = @(
        @('electron', (Join-Path $repository 'node_modules/electron.exe')),
        @('node', (Join-Path $repository 'tools/node.exe')),
        @('nemu-core', (Join-Path $repository 'core/target/nemu-core.exe')),
        @('node', (Join-Path ($repository + '-other') 'node.exe')),
        @('electron', (Join-Path $env:RUNNER_TEMP 'private-app/electron.exe')),
        @('private-fixture-app', (Join-Path $repository 'private-fixture-app.exe')),
        @('node', $null)
    )
    for ($i = 0; $i -lt $rows.Count; $i++) {
        [pscustomobject]@{ Id = $i + 1; ProcessName = $rows[$i][0]; Path = $rows[$i][1]
            SessionId = 1; WorkingSet64 = 10; PrivateMemorySize64 = 5; CPU = 2
            MainWindowTitle = 'private-fixture-title'; CommandLine = 'private-fixture-command' }
    }
}
function Get-NetTCPConnection {
    param($State, $ErrorAction)
    if ($script:denyPorts) { throw 'private-fixture-error-with-path' }
    [pscustomobject]@{ LocalAddress = '192.0.2.1'; LocalPort = 12345; OwningProcess = 5 }
}
$sample = Get-PublicWindowsSnapshot -RepositoryRoot $repository
if ($sample.processCount -ne 7 -or $sample.resources.workingSetBytes -ne 70 -or
    $sample.resources.privateMemoryBytes -ne 35 -or $sample.resources.cpuSeconds -ne 14) {
    throw 'Resource totals must include all processes'
}
if (($sample.processes.Id -join ',') -ne '1,2,3' -or $sample.listeningTcpCount -ne 1) {
    throw 'Only checkout-owned named processes may be exported; TCP count must remain available'
}
$json = $sample | ConvertTo-Json -Depth 6
if ($json -match 'private-fixture|private-app|checkout-other|192\.0\.2\.1|12345|OwningProcess|MainWindowTitle|CommandLine|"Path"|"sessions"') {
    throw 'Private fixture information escaped the snapshot projection'
}
$script:denyPorts = $true
$sample = Get-PublicWindowsSnapshot -RepositoryRoot $repository
if ($sample.portsError -ne 'unavailable' -or ($sample | ConvertTo-Json -Depth 6) -match 'private-fixture') {
    throw 'Collection errors must not disclose raw messages'
}
Write-Output 'PASS: totals include background load; unrelated names, paths, titles, ports and error text excluded.'

. (Join-Path $PSScriptRoot 'windows-evidence-text.ps1')
$savedProfile = $env:USERPROFILE
try {
    $env:USERPROFILE = 'C:\Users\private-fixture-user'
    $lines = @('error C:\Users\private-fixture-user\tool\file.rs:42',
        'error C:/Users/private-fixture-user/tool/file.rs:42', 'test result: FAILED') | Protect-WindowsEvidenceText
    if (($lines -join "`n") -match 'private-fixture-user' -or $lines[2] -ne 'test result: FAILED' -or
        $lines[0] -notmatch '<user-profile>.*file.rs:42') { throw 'Log redaction lost diagnostics or leaked a profile' }
    & pwsh -NoProfile -Command 'Write-Output "native failure"; exit 23' | Protect-WindowsEvidenceText | Out-Null
    if ($LASTEXITCODE -ne 23) { throw 'Log redaction changed native command exit status' }
} finally { $env:USERPROFILE = $savedProfile }
Write-Output 'PASS: log paths redacted; failure text, source location and native exit status preserved.'
