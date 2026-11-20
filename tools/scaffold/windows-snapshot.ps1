# Only this projection may cross the public evidence boundary (AGENTS section 6).
function Get-PublicWindowsSnapshot {
    param([string]$RepositoryRoot)
    $all = @(Get-Process)
    $prefix = [IO.Path]::GetFullPath($RepositoryRoot).TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
    $project = @($all | Where-Object {
        # A name alone is not ownership: unrelated Electron/Node apps must stay private.
        if ($_.ProcessName -notmatch '^(electron|node|nodejs|nemu-core|Runner\.(Listener|Worker))$') { return $false }
        try {
            $path = $_.Path
            return $path -and [IO.Path]::GetFullPath($path).StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)
        } catch { return $false }
    } | Select-Object Id, ProcessName, SessionId)
    $sample = [ordered]@{
        at = [DateTime]::UtcNow.ToString('o')
        interactive = [Environment]::UserInteractive
        sessionId = (Get-Process -Id $PID).SessionId
        processCount = $all.Count
        resources = @{
            workingSetBytes = ($all | Measure-Object WorkingSet64 -Sum).Sum
            privateMemoryBytes = ($all | Measure-Object PrivateMemorySize64 -Sum).Sum
            cpuSeconds = ($all | Measure-Object CPU -Sum).Sum
        }
        processScope = 'Known project executable names with executable path inside this checkout; inaccessible paths omitted'
        processes = $project
        lockState = 'Unknown: interactive/session Active does not prove unlocked'
    }
    try {
        # Keep only a count, never addresses, port numbers or background process ownership.
        $sample.listeningTcpCount = @(Get-NetTCPConnection -State Listen -ErrorAction Stop).Count
    } catch { $sample.portsError = 'unavailable' }
    return $sample
}
