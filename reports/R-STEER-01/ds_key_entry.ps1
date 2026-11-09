# User-operated hidden input; never persists the key or changes user/system env.
[CmdletBinding()]
param()
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$queue = 'D:/Dev/cache/R-STEER-01/ds-flash-pilot-01'
if (Test-Path -LiteralPath $queue) { throw 'Existing key-entry session; preserve it' }
New-Item -ItemType Directory -Path $queue | Out-Null
$Host.UI.RawUI.WindowTitle = 'R-STEER-01 DeepSeek Flash KEY - process only'
$entrySha = (Get-FileHash -LiteralPath $PSCommandPath).Hash.ToLowerInvariant()
$secret = $null
try {
    Write-Host 'R-STEER-01: DeepSeek Flash pilot, maximum 20 requests; full round requires later user confirmation.'
    Write-Host 'Enter the key below (hidden). It stays in this process and its scoring child only.'
    Write-Host 'Do not paste the key into chat or save it in a file. Closing this window cancels.'
    $secret = Read-Host 'DEEPSEEK_API_KEY' -AsSecureString
    if ($secret.Length -eq 0) { throw 'Empty key' }
    [ordered]@{status='key_entered_in_memory'; recordedUtc=[DateTimeOffset]::UtcNow.ToString('o'); entrySha256=$entrySha; model='deepseek-flash'; secretPersisted=$false} | ConvertTo-Json | Set-Content (Join-Path $queue 'entry-status.json') -Encoding utf8NoBOM
    Write-Host 'Key received locally. Waiting for the frozen, pushed pilot preparation; no API request yet.'
    # One hour is an engineering handoff bound, not a research criterion.
    $deadline = [DateTimeOffset]::UtcNow.AddHours(1)
    $readyPath = Join-Path $queue 'ready.json'
    while (!(Test-Path -LiteralPath $readyPath)) {
        if ([DateTimeOffset]::UtcNow -gt $deadline) { throw 'Preparation handoff timed out; no API request' }
        Start-Sleep -Seconds 2
    }
    $ready = Get-Content -Raw -LiteralPath $readyPath | ConvertFrom-Json
    if ($ready.model -ne 'deepseek-flash' -or $ready.entrySha256 -ne $entrySha -or $ready.phase -ne 'pilot') { throw 'Frozen handoff mismatch' }
    $env:DEEPSEEK_API_KEY = [Net.NetworkCredential]::new('', $secret).Password
    $secret.Dispose()
    $secret = $null
    Write-Host 'Starting the frozen 20-item pilot. No full-round run will start from this window.'
    & 'D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe' -B (Join-Path $PSScriptRoot 'ds_judge.py') pilot --session $queue --expected-head $ready.head
    $code = $LASTEXITCODE
    Write-Host "Pilot process finished (exit $code). Results contain no key."
} catch {
    # Do not stringify unexpected exceptions: providers can echo credentials.
    Write-Host 'Key-entry/pilot stopped. No automatic retry. See the non-secret session status.'
    [ordered]@{status='entry_or_pilot_stopped'; recordedUtc=[DateTimeOffset]::UtcNow.ToString('o')} | ConvertTo-Json | Set-Content (Join-Path $queue 'entry-stopped.json') -Encoding utf8NoBOM
} finally {
    Remove-Item Env:DEEPSEEK_API_KEY -ErrorAction SilentlyContinue
    if ($null -ne $secret) { $secret.Dispose() }
}
Read-Host 'Press Enter to close' | Out-Null
