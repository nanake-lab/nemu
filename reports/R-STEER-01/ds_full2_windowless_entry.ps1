# User-operated process-only key handoff to the frozen full-504 supervisor.
[CmdletBinding()]
param()
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$queue = 'D:/Dev/cache/R-STEER-01/ds-flash-full-02-windowless'
$marker = [IO.File]::Open((Join-Path $queue 'entry-started'), [IO.FileMode]::CreateNew)
$marker.Dispose()
$Host.UI.RawUI.WindowTitle = 'R-STEER-01 Flash HIGH V2 remaining434 KEY - process only'
$secret = $null
try {
    $gate = Get-Content -Raw -LiteralPath (Join-Path $queue 'execute.json') | ConvertFrom-Json
    if ($gate.phase -ne 'full504_high_v2_windowless' -or $gate.head -notmatch '^[0-9a-f]{40}$') { throw 'Invalid frozen handoff' }
    Write-Host 'Flash HIGH: 434 REMAINING V2 ratings; 70 preserved; actual worker windows disabled, $5 cumulative hard cap including all pilots and unknown reserves.'
    Write-Host 'Enter DEEPSEEK_API_KEY below (hidden). This window closes after detached handoff.'
    $secret = Read-Host 'DEEPSEEK_API_KEY' -AsSecureString
    if ($secret.Length -eq 0) { throw 'Empty key' }
    $env:DEEPSEEK_API_KEY = [Net.NetworkCredential]::new('', $secret).Password
    $secret.Dispose()
    $secret = $null
    & 'C:/Users/nanake/AppData/Local/Programs/Python/Python312/python.exe' -B (Join-Path $PSScriptRoot 'ds_full2_windowless_run.py') launch --head $gate.head
    [ordered]@{status='handoff_returned'; exit_code=$LASTEXITCODE; utc=[DateTimeOffset]::UtcNow.ToString('o')} | ConvertTo-Json | Set-Content (Join-Path $queue 'entry-result.json') -Encoding utf8NoBOM
} catch {
    [ordered]@{status='entry_stopped'; utc=[DateTimeOffset]::UtcNow.ToString('o')} | ConvertTo-Json | Set-Content (Join-Path $queue 'entry-stop.json') -Encoding utf8NoBOM
} finally {
    Remove-Item Env:DEEPSEEK_API_KEY -ErrorAction SilentlyContinue
    if ($null -ne $secret) { $secret.Dispose() }
}
