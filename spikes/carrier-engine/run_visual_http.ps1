param([string]$ScriptName = 'visual_http_benchmark.py', [string]$LogPrefix = 'item-4')
$ErrorActionPreference = 'Stop'
$cache = 'D:/Dev/cache/P0-12/gpu-01'
$window = Get-Content (Join-Path $cache 'window.json') -Raw | ConvertFrom-Json
$remaining = ([DateTimeOffset]::Parse($window.deadlineUtc) - [DateTimeOffset]::UtcNow).TotalMilliseconds
if ($remaining -le 0) { throw 'Three-hour GPU deadline reached' }
if (@(Get-Process | Where-Object {$_.ProcessName -in @('Runner.Listener','Runner.Worker','blender')}).Count) { throw 'Quiet window absent' }
$env:PYTHONDONTWRITEBYTECODE = '1'
$process = Start-Process -FilePath 'D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe' -ArgumentList @('-u', (Join-Path $PSScriptRoot $ScriptName)) `
    -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $cache "$LogPrefix.stdout.log") -RedirectStandardError (Join-Path $cache "$LogPrefix.stderr.log")
$timedOut = !$process.WaitForExit([int][Math]::Min($remaining, [int]::MaxValue))
if ($timedOut) { $process.Kill($true); $process.WaitForExit() }
$process.Refresh()
[ordered]@{exitCode=$process.ExitCode; timedOut=$timedOut; endedUtc=[DateTimeOffset]::UtcNow.ToString('o')} |
    ConvertTo-Json | Set-Content (Join-Path $cache "$LogPrefix-result.json") -Encoding utf8NoBOM
if ($timedOut -or $process.ExitCode -ne 0) { throw 'Measurement stopped; preserve logs, no retry' }
