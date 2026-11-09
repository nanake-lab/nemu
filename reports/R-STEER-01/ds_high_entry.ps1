# User-operated hidden input, new high-only session; never persists credentials.
[CmdletBinding()]
param()
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$queue = 'D:/Dev/cache/R-STEER-01/ds-flash-high-01'
$started = Join-Path $queue 'entry-started'
$marker = [IO.File]::Open($started, [IO.FileMode]::CreateNew)
$marker.Dispose()
$Host.UI.RawUI.WindowTitle = 'R-STEER-01 Flash HIGH KEY - process only'
$secret = $null
try {
    Write-Host 'Flash HIGH: the same 20 items only. No full round. Window closes after completion.'
    Write-Host 'Enter DEEPSEEK_API_KEY below (hidden); never paste it into chat or a file.'
    $secret = Read-Host 'DEEPSEEK_API_KEY' -AsSecureString
    if ($secret.Length -eq 0) { throw 'Empty key' }
    [ordered]@{status='key_entered_in_memory'; utc=[DateTimeOffset]::UtcNow.ToString('o'); secretPersisted=$false} | ConvertTo-Json | Set-Content (Join-Path $queue 'entry-status.json') -Encoding utf8NoBOM
    Write-Host 'Waiting for pushed registration and fresh official price check. No API request yet.'
    $deadline = [DateTimeOffset]::UtcNow.AddHours(1) # Engineering handoff bound.
    $gate = Join-Path $queue 'execute.json'
    while (!(Test-Path -LiteralPath $gate)) {
        if ([DateTimeOffset]::UtcNow -gt $deadline) { throw 'Handoff timeout' }
        Start-Sleep -Seconds 2
    }
    $execute = Get-Content -Raw -LiteralPath $gate | ConvertFrom-Json
    if ($execute.phase -ne 'high_twenty_only' -or $execute.head -notmatch '^[0-9a-f]{40}$') { throw 'Invalid handoff' }
    $env:DEEPSEEK_API_KEY = [Net.NetworkCredential]::new('', $secret).Password
    $secret.Dispose()
    $secret = $null
    & 'D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe' -B (Join-Path $PSScriptRoot 'ds_high_run.py') --expected-head $execute.head
    [ordered]@{status='high_process_exited'; exit_code=$LASTEXITCODE; utc=[DateTimeOffset]::UtcNow.ToString('o')} | ConvertTo-Json | Set-Content (Join-Path $queue 'entry-result.json') -Encoding utf8NoBOM
} catch {
    [ordered]@{status='entry_stopped_no_retry'; utc=[DateTimeOffset]::UtcNow.ToString('o')} | ConvertTo-Json | Set-Content (Join-Path $queue 'entry-stop.json') -Encoding utf8NoBOM
} finally {
    Remove-Item Env:DEEPSEEK_API_KEY -ErrorAction SilentlyContinue
    if ($null -ne $secret) { $secret.Dispose() }
}
