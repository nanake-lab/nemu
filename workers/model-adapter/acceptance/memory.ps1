param([Parameter(Mandatory)][string]$OutputPath, [Parameter(Mandatory)][string]$StopPath,
      [Parameter(Mandatory)][int]$MeasurementRoot)
$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $OutputPath) { throw 'Use a new evidence file' }
$writer = [IO.StreamWriter]::new($OutputPath, $false, [Text.UTF8Encoding]::new($false))
try {
    while (!(Test-Path -LiteralPath $StopPath)) {
        $tree = @(Get-CimInstance Win32_Process -Property ProcessId,ParentProcessId)
        $owned = [Collections.Generic.HashSet[int]]::new()
        [void]$owned.Add($MeasurementRoot)
        do {
            $old = $owned.Count
            foreach ($p in $tree) {
                if ($owned.Contains($p.ParentProcessId)) { [void]$owned.Add($p.ProcessId) }
            }
        } until ($owned.Count -eq $old)
        $dedicated = [uint64]0
        $shared = [uint64]0
        $instances = 0
        foreach ($row in Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUProcessMemory) {
            if ($row.Name -match '^pid_(\d+)_' -and $owned.Contains([int]$Matches[1])) {
                $dedicated += [uint64]$row.DedicatedUsage
                $shared += [uint64]$row.SharedUsage
                $instances++
            }
        }
        $writer.WriteLine((@{utc=[DateTimeOffset]::UtcNow.ToString('o'); instances=$instances;
                             dedicated=$dedicated; shared=$shared} | ConvertTo-Json -Compress))
        $writer.Flush()
        Start-Sleep -Milliseconds 200
    }
} finally { $writer.Dispose() }
