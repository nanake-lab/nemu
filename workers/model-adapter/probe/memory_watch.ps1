param(
    [Parameter(Mandatory)][string]$OutputPath,
    [Parameter(Mandatory)][string]$StopPath,
    [Parameter(Mandatory)][int[]]$RootProcessIds
)
$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $OutputPath) { throw 'Use a new evidence path' }
$owned = [System.Collections.Generic.HashSet[int]]::new()
foreach ($rootProcessId in $RootProcessIds) { [void]$owned.Add($rootProcessId) }
$writer = [System.IO.StreamWriter]::new($OutputPath, $false, [System.Text.UTF8Encoding]::new($false))
try {
    while (!(Test-Path -LiteralPath $StopPath)) {
        $started = [DateTimeOffset]::UtcNow
        $processes = @(Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId,Name,CreationDate)
        do {
            $added = 0
            foreach ($process in $processes) {
                if ($owned.Contains([int]$process.ParentProcessId) -and $owned.Add([int]$process.ProcessId)) { $added++ }
            }
        } while ($added -gt 0)
        $memory = @(Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUProcessMemory |
            Select-Object Name,DedicatedUsage,SharedUsage,TotalCommitted,LocalUsage,NonLocalUsage)
        $row = [ordered]@{
            startedUtc = $started.ToString('o')
            endedUtc = [DateTimeOffset]::UtcNow.ToString('o')
            rootProcessIds = $RootProcessIds
            ownedProcesses = @($processes | Where-Object { $owned.Contains([int]$_.ProcessId) })
            counters = $memory
            scope = 'WDDM process counters, sampled; not an exact continuous peak'
        }
        $writer.WriteLine(($row | ConvertTo-Json -Depth 6 -Compress))
        $writer.Flush()
        Start-Sleep -Milliseconds 200
    }
} catch {
    $writer.WriteLine((@{error=$_.Exception.Message; at=[DateTimeOffset]::UtcNow.ToString('o')} | ConvertTo-Json -Compress))
    throw
} finally { $writer.Dispose() }
