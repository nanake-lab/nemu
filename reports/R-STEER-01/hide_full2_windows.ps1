# Operational UI-only workaround: hide this supervisor's exact child windows.
# Does not alter request execution, deadlines, prompts, ratings or system settings.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$sessionRoot = 'D:/Dev/cache/R-STEER-01/ds-flash-full-02'
$binding = Get-Content -Raw -LiteralPath (Join-Path $sessionRoot 'run/binding.json') | ConvertFrom-Json
$supervisorId = [int]$binding.pid
if ($supervisorId -ne 42460 -or $binding.head -ne '144e6b1a364bf228b696f1d2ff5973c3ffafa3f9') { throw 'Unexpected supervisor' }
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class Full2WindowVisibility {
  [DllImport("user32.dll")] public static extern bool ShowWindowAsync(IntPtr window, int command);
}
'@
$seenWindows = [Collections.Generic.HashSet[long]]::new()
$log = Join-Path $sessionRoot 'window-visibility.jsonl'
while ($null -ne (Get-Process -Id $supervisorId -ErrorAction SilentlyContinue)) {
    if (Test-Path -LiteralPath (Join-Path $sessionRoot 'run/result.json')) { break }
    $children = Get-CimInstance Win32_Process -Filter "ParentProcessId=$supervisorId AND Name='python.exe'" -Property ProcessId
    foreach ($child in $children) {
        $process = Get-Process -Id $child.ProcessId -ErrorAction SilentlyContinue
        if ($null -eq $process) { continue }
        $window = $process.MainWindowHandle
        if ($window -ne [IntPtr]::Zero -and $seenWindows.Add($window.ToInt64())) {
            $hidden = [Full2WindowVisibility]::ShowWindowAsync($window, 0)
            [ordered]@{utc=[DateTimeOffset]::UtcNow.ToString('o'); event='hide_owned_child_window'; supervisor_pid=$supervisorId; child_pid=$process.Id; window=$window.ToInt64(); result=$hidden} |
                ConvertTo-Json -Compress | Add-Content -LiteralPath $log -Encoding utf8NoBOM
        }
    }
    Start-Sleep -Milliseconds 250
}
[ordered]@{utc=[DateTimeOffset]::UtcNow.ToString('o'); event='visibility_helper_ended'} | ConvertTo-Json -Compress | Add-Content -LiteralPath $log -Encoding utf8NoBOM
