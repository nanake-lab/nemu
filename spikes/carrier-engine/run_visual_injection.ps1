param([string]$OutputDirectory = 'D:/Dev/cache/P0-12/gpu-01')
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$runner = @(Get-Process | Where-Object {$_.ProcessName -in @('Runner.Listener', 'Runner.Worker', 'blender')})
if ($runner.Count) { throw 'GPU quiet window absent; no GPU started' }
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Preserve previous GPU measurements' }
New-Item -ItemType Directory -Path $OutputDirectory | Out-Null
$start = [DateTimeOffset]::UtcNow
$deadline = $start.AddHours(3)
[ordered]@{startedUtc=$start.ToString('o'); deadlineUtc=$deadline.ToString('o'); runnerProcesses=@(); hardLimitSeconds=10800} |
    ConvertTo-Json | Set-Content (Join-Path $OutputDirectory 'window.json') -Encoding utf8NoBOM
$exe = 'D:/Dev/cache/P0-12/build-visual-01/injection_probe.exe'
foreach ($stage in @(1, 2, 3)) {
    $remaining = ($deadline - [DateTimeOffset]::UtcNow).TotalMilliseconds
    if ($remaining -le 0) { throw 'GPU hard deadline reached' }
    $arguments = @('D:/Dev/cache/P0-10/llama.cpp/b11669', 'D:/Dev/cache/P0-12/gguf/carrier-Q4_K_M.gguf',
        (Join-Path $OutputDirectory "item-$stage"), (Join-Path $root 'reports/P0-12/inputs'), "$stage", '--quiet-window-confirmed')
    [ordered]@{executable=$exe; arguments=$arguments; startedUtc=[DateTimeOffset]::UtcNow.ToString('o')} |
        ConvertTo-Json | Set-Content (Join-Path $OutputDirectory "item-$stage-command.json") -Encoding utf8NoBOM
    $process = Start-Process -FilePath $exe -ArgumentList $arguments -WindowStyle Hidden -PassThru `
        -RedirectStandardOutput (Join-Path $OutputDirectory "item-$stage.stdout.log") `
        -RedirectStandardError (Join-Path $OutputDirectory "item-$stage.stderr.log")
    $timedOut = !$process.WaitForExit([int][Math]::Min($remaining, [int]::MaxValue))
    if ($timedOut) { $process.Kill($true); $process.WaitForExit() }
    $process.Refresh()
    [ordered]@{exitCode=$process.ExitCode; timedOut=$timedOut; endedUtc=[DateTimeOffset]::UtcNow.ToString('o')} |
        ConvertTo-Json | Set-Content (Join-Path $OutputDirectory "item-$stage-result.json") -Encoding utf8NoBOM
    if ($timedOut -or $process.ExitCode -ne 0) { throw "Item $stage stopped; preserve evidence and do not retry" }
}
