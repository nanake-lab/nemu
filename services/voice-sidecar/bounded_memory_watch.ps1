param(
    [Parameter(Mandatory)][string]$OutputPath,
    [Parameter(Mandatory)][string]$StopPath,
    [Parameter(Mandatory)][int[]]$RootProcessIds
)
$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $OutputPath) { throw 'Use a new evidence path' }
$writer = [System.IO.StreamWriter]::new($OutputPath, $false, [System.Text.UTF8Encoding]::new($false))
$processSnapshot = @()
$owned = [System.Collections.Generic.HashSet[int]]::new()
$lastTreeRefresh = [DateTimeOffset]::MinValue

function Get-CimRows {
    param([Parameter(Mandatory)][string]$ClassName,
          [Parameter(Mandatory)][string[]]$Properties)
    for ($attempt = 1; $attempt -le 3; $attempt++) {
        try {
            return @(Get-CimInstance -ClassName $ClassName -ErrorAction Stop |
                Select-Object -Property $Properties)
        } catch {
            if ($attempt -eq 3) { throw }
            Start-Sleep -Milliseconds ($attempt * 200)
        }
    }
}

$observations = 0
try {
    while (!(Test-Path -LiteralPath $StopPath)) {
        $started = [DateTimeOffset]::UtcNow
        if (($started - $lastTreeRefresh).TotalSeconds -ge 5) {
            $processSnapshot = @(Get-CimRows -ClassName 'Win32_Process' `
                -Properties @('ProcessId','ParentProcessId','Name','CreationDate'))
            # Only active descendants are retained; refresh less often than GPU
            # counters so transient helper PIDs cannot grow state or saturate CIM.
            $owned = [System.Collections.Generic.HashSet[int]]::new()
            foreach ($rootProcessId in $RootProcessIds) { [void]$owned.Add($rootProcessId) }
            do {
                $added = 0
                foreach ($process in $processSnapshot) {
                    if ($owned.Contains([int]$process.ParentProcessId) -and $owned.Add([int]$process.ProcessId)) {
                        $added++
                    }
                }
            } while ($added -gt 0)
            $lastTreeRefresh = $started
        }
        $memory = @(Get-CimRows -ClassName 'Win32_PerfFormattedData_GPUPerformanceCounters_GPUProcessMemory' `
            -Properties @('Name','DedicatedUsage','SharedUsage','TotalCommitted','LocalUsage','NonLocalUsage'))
        $row = [ordered]@{
            startedUtc = $started.ToString('o')
            endedUtc = [DateTimeOffset]::UtcNow.ToString('o')
            rootProcessIds = $RootProcessIds
            processTreeSampleUtc = $lastTreeRefresh.ToString('o')
            processTreeRefreshSeconds = 5
            ownedProcesses = @($processSnapshot | Where-Object { $owned.Contains([int]$_.ProcessId) })
            counters = $memory
            scope = 'WDDM process counters, sampled; not an exact continuous peak'
        }
        $writer.WriteLine(($row | ConvertTo-Json -Depth 6 -Compress))
        $writer.Flush()
        $observations++
        if (($observations % 25) -eq 0) {
            $memory = $null
            $row = $null
            [GC]::Collect()
        }
        Start-Sleep -Milliseconds 200
    }
} catch {
    $writer.WriteLine((@{error=$_.Exception.Message; at=[DateTimeOffset]::UtcNow.ToString('o')} | ConvertTo-Json -Compress))
    $writer.Flush()
    throw
} finally { $writer.Dispose() }
