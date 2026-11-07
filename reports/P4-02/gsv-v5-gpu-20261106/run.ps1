$ErrorActionPreference='Stop'
. "$PSScriptRoot/monitor-core.ps1"
$out=$PSScriptRoot
$launcher=$null; $actualId=0; $owned=[System.Collections.Generic.HashSet[int]]::new()
try {
 $pre=@(Get-CimInstance Win32_Process)
 $bad=@(Get-ExternalHeavyProcesses $pre $owned)
 $active=@(Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine | Where-Object UtilizationPercentage -gt 0 | Select-Object Name,UtilizationPercentage)
 $engines=@(Get-ExternalHeavyEngines $active $owned)
 [ordered]@{utc=[DateTimeOffset]::UtcNow.ToString('o');processes=@($pre | Select-Object ProcessId,ParentProcessId,Name);forbidden=@($bad | Select-Object ProcessId,Name);engines=$active;heavy_engines=$engines;free_disk_bytes=(Get-PSDrive D).Free} | ConvertTo-Json -Depth 5 | Set-Content "$out/preflight.json" -Encoding utf8
 nvidia-smi | Set-Content "$out/nvidia-before.txt" -Encoding utf8
 if($bad.Count -or $engines.Count) { throw 'Preflight runner/heavy load detected' }
 $launcher=Start-Process 'D:\Dev\cache\p4-02\gsv-v5-venv\Scripts\python.exe' -ArgumentList @('-u',"$out/measure.py") -WindowStyle Hidden -RedirectStandardOutput "$out/stdout.log" -RedirectStandardError "$out/stderr.log" -PassThru
 $ready=Wait-ReadyEvent "$out/events.jsonl" $launcher
 $actualId=[int]$ready.pid
 $snapshot=@(Get-CimInstance Win32_Process)
 $chain=Confirm-ParentChain $actualId $launcher.Id $snapshot
 $birth=($snapshot | Where-Object ProcessId -eq $actualId).CreationDate
 $owned=Get-OwnedIds $actualId $snapshot
 [ordered]@{launcher=$launcher.Id;actual=$actualId;parent_chain=$chain;creation=$birth;processes=@($snapshot | Where-Object { $owned.Contains([int]$_.ProcessId) -or $_.ProcessId -eq $launcher.Id } | Select-Object ProcessId,ParentProcessId,Name,CreationDate)} | ConvertTo-Json -Depth 5 | Set-Content "$out/ownership.json" -Encoding utf8
 $lastCheck=[DateTimeOffset]::MinValue
 while($true) {
  $now=[DateTimeOffset]::UtcNow
  $snapshot=@(Get-CimInstance Win32_Process)
  $root=@($snapshot | Where-Object ProcessId -eq $actualId)
  if(!$root.Count) { break }
  if($root[0].CreationDate -ne $birth) { throw 'Actual PID reused' }
  $owned=Get-OwnedIds $actualId $snapshot
  $memory=@(Get-OwnedMemory $owned)
  $counters=@(Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUProcessMemory | Select-Object Name,DedicatedUsage,SharedUsage)
  [ordered]@{utc=$now.ToUnixTimeMilliseconds()/1000.0;actual=$actualId;owned=@($owned);process_memory=$memory;rss=($memory | Measure-Object rss -Sum).Sum;private=($memory | Measure-Object private -Sum).Sum;counters=$counters} | ConvertTo-Json -Depth 5 -Compress | Add-Content "$out/memory.jsonl" -Encoding utf8
  if(($now-$lastCheck).TotalSeconds -ge 5) {
   $bad=@(Get-ExternalHeavyProcesses $snapshot $owned)
   $active=@(Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine | Where-Object UtilizationPercentage -gt 0 | Select-Object Name,UtilizationPercentage)
   $engines=@(Get-ExternalHeavyEngines $active $owned)
   [ordered]@{utc=$now.ToUnixTimeMilliseconds()/1000.0;actual=$actualId;owned=@($owned);processes=@($snapshot | Select-Object ProcessId,ParentProcessId,Name);active_engines=$active;forbidden=@($bad | Select-Object ProcessId,Name);external_heavy_engines=$engines} | ConvertTo-Json -Depth 5 -Compress | Add-Content "$out/load-guard.jsonl" -Encoding utf8
   if($bad.Count -or $engines.Count) { throw 'Heavy-load interference detected' }
   $lastCheck=$now
  }
  if(!(Test-Path "$out/monitor-ready")) { 'ready' | Set-Content "$out/monitor-ready" }
  Start-Sleep -Milliseconds 500
 }
 $launcher.WaitForExit()
 $launcher.ExitCode | Set-Content "$out/exit.txt"
 if($launcher.ExitCode -ne 0) { throw "GPU script failed: $($launcher.ExitCode)" }
 $last=Get-Content "$out/events.jsonl" -Tail 1 | ConvertFrom-Json
 if($last.event -ne 'complete') { throw 'No complete event' }
} catch {
 $_ | Out-String | Set-Content "$out/monitor-failure.txt"
 'stop' | Set-Content "$out/stop"
 # Stop only this run's verified tree. Do not operate any external process.
 if($actualId -gt 0) {
  $snapshot=@(Get-CimInstance Win32_Process)
  $root=@($snapshot | Where-Object ProcessId -eq $actualId)
  if($root.Count -and $root[0].CreationDate -eq $birth) {
   $owned=Get-OwnedIds $actualId $snapshot
   foreach($id in $owned) { Stop-Process -Id $id -ErrorAction SilentlyContinue }
  }
 }
 if($launcher -and !$launcher.HasExited) { Stop-Process -Id $launcher.Id -ErrorAction SilentlyContinue }
 throw
} finally {
 nvidia-smi | Set-Content "$out/nvidia-after.txt" -Encoding utf8
}
