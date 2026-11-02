param([string]$ScriptName = 'visual_http_benchmark.py', [string]$LogPrefix = 'item-4')
$ErrorActionPreference = 'Stop'
$cache = 'D:/Dev/cache/P0-12/gpu-02'
if (!(Test-Path -LiteralPath $cache)) {
    New-Item -ItemType Directory -Path $cache | Out-Null
    $started = [DateTimeOffset]::UtcNow
    # Count the previous 156.257-second window against the user's total three-hour cap.
    [ordered]@{startedUtc=$started.ToString('o'); deadlineUtc=$started.AddSeconds(10800 - 156.257).ToString('o'); priorWindowSeconds=156.257; totalLimitSeconds=10800} |
        ConvertTo-Json | Set-Content (Join-Path $cache 'window.json') -Encoding utf8NoBOM
}
$window = Get-Content (Join-Path $cache 'window.json') -Raw | ConvertFrom-Json
$remaining = ([DateTimeOffset]::Parse($window.deadlineUtc) - [DateTimeOffset]::UtcNow).TotalMilliseconds
if ($remaining -le 0) { throw 'Three-hour GPU deadline reached' }
if (@(Get-Process | Where-Object {$_.ProcessName -in @('Runner.Listener','Runner.Worker','blender')}).Count) { throw 'Quiet window absent' }
$env:PYTHONDONTWRITEBYTECODE = '1'
$process = Start-Process -FilePath 'D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe' -ArgumentList @('-u', (Join-Path $PSScriptRoot $ScriptName)) `
    -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $cache "$LogPrefix.stdout.log") -RedirectStandardError (Join-Path $cache "$LogPrefix.stderr.log")
$timedOut = $false
$quietFailure = $false
while ($true) {
    $left = ([DateTimeOffset]::Parse($window.deadlineUtc) - [DateTimeOffset]::UtcNow).TotalMilliseconds
    if ($left -le 0) { $timedOut=$true; $process.Kill($true); $process.WaitForExit(); break }
    if ($process.WaitForExit([int][Math]::Min(5000, $left))) { break }
    $now = [DateTimeOffset]::UtcNow
    $blockers = @(Get-Process -ErrorAction Stop | Where-Object {$_.ProcessName -in @('Runner.Listener','Runner.Worker','blender')} | Select-Object ProcessName,Id)
    [ordered]@{checkedUtc=$now.ToString('o'); blockers=$blockers} | ConvertTo-Json -Compress -Depth 4 |
        Add-Content (Join-Path $cache "$LogPrefix-quiet.jsonl") -Encoding utf8NoBOM
    $quietFailure = $blockers.Count -gt 0
    $timedOut = $now -ge [DateTimeOffset]::Parse($window.deadlineUtc)
    if ($timedOut -or $quietFailure) { $process.Kill($true); $process.WaitForExit(); break }
}
$process.Refresh()
[ordered]@{exitCode=$process.ExitCode; timedOut=$timedOut; quietFailure=$quietFailure; endedUtc=[DateTimeOffset]::UtcNow.ToString('o')} |
    ConvertTo-Json | Set-Content (Join-Path $cache "$LogPrefix-result.json") -Encoding utf8NoBOM
if ($timedOut -or $quietFailure -or $process.ExitCode -ne 0) { throw 'Measurement stopped; preserve logs, no retry' }
