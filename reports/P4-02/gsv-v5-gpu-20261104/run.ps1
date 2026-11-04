$ErrorActionPreference='Stop'
$out = $PSScriptRoot
$python = 'D:\Dev\cache\p4-02\gsv-v5-venv\Scripts\python.exe'
$bench = $null
try {
 $pre = @(Get-CimInstance Win32_Process)
 $forbidden = @($pre | Where-Object { $_.Name -match '^(Runner\.(Listener|Worker)|llama-server|llama-cli|ollama|blender|vlc|mpv|ffplay|GenshinImpact|StarRail|ZenlessZoneZero|League of Legends|VALORANT.*|cs2|dota2)\.exe$' -or ($_.Name -match '^python' -and $_.CommandLine -match 'llama|sovits|comfyui|stable.diffusion|vllm|text-generation|inference') })
 if ($forbidden.Count) { throw ('Preflight heavy process: '+(($forbidden | Select-Object Name,ProcessId | ConvertTo-Json -Compress))) }
 $activeVideo = @(Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine | Where-Object { $_.Name -match 'VideoDecode|VideoProcessing' -and $_.UtilizationPercentage -gt 0 })
 if ($activeVideo.Count) { throw 'Active video engine before measurement' }
 $bench = Start-Process -FilePath $python -ArgumentList @('-u',"$out\measure.py") -WindowStyle Hidden -RedirectStandardOutput "$out\stdout.log" -RedirectStandardError "$out\stderr.log" -PassThru
 $bench.Id | Set-Content "$out/bench.pid"
 $lastCheck=[DateTimeOffset]::MinValue
 while (!$bench.HasExited) {
  $now=[DateTimeOffset]::UtcNow
  $mem = @(Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUProcessMemory | Select-Object Name,DedicatedUsage,SharedUsage)
  $process = Get-Process -Id $bench.Id -ErrorAction Stop
  [ordered]@{utc=$now.ToUnixTimeMilliseconds()/1000.0;pid=$bench.Id;rss=$process.WorkingSet64;private=$process.PrivateMemorySize64;counters=$mem} | ConvertTo-Json -Depth 4 -Compress | Add-Content "$out/memory.jsonl" -Encoding utf8
  if (($now-$lastCheck).TotalSeconds -ge 5) {
   $processes=@(Get-CimInstance Win32_Process)
   $bad=@($processes | Where-Object { $_.ProcessId -ne $bench.Id -and ($_.Name -match '^(Runner\.(Listener|Worker)|llama-server|llama-cli|ollama|blender|vlc|mpv|ffplay|GenshinImpact|StarRail|ZenlessZoneZero|League of Legends|VALORANT.*|cs2|dota2)\.exe$' -or ($_.Name -match '^python' -and $_.CommandLine -match 'llama|sovits|comfyui|stable.diffusion|vllm|text-generation|inference')) })
   $engines=@(Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine | Where-Object { $_.UtilizationPercentage -gt 0 } | Select-Object Name,UtilizationPercentage)
   $heavy=@($engines | Where-Object { $_.Name -notmatch ('^pid_'+$bench.Id+'_') -and $_.Name -match 'VideoDecode|VideoProcessing|engtype_Compute|engtype_Cuda' })
   [ordered]@{utc=$now.ToUnixTimeMilliseconds()/1000.0;processes=@($processes | Select-Object ProcessId,Name);active_engines=$engines;forbidden=@($bad | Select-Object ProcessId,Name);external_heavy_engines=$heavy} | ConvertTo-Json -Depth 4 -Compress | Add-Content "$out/load-guard.jsonl" -Encoding utf8
   if ($bad.Count -or $heavy.Count) { throw 'Heavy-load interference detected; see load-guard.jsonl' }
   $lastCheck=$now
  }
  if (!(Test-Path "$out/monitor-ready")) { 'ready' | Set-Content "$out/monitor-ready" }
  Start-Sleep -Milliseconds 500
  $bench.Refresh()
 }
 $bench.WaitForExit()
 $bench.ExitCode | Set-Content "$out/exit.txt"
 if ($bench.ExitCode -ne 0) { throw "GPU script failed: $($bench.ExitCode)" }
} catch {
 $_.Exception.Message | Set-Content "$out/monitor-failure.txt"
 'stop' | Set-Content "$out/stop"
 if ($bench -and !$bench.HasExited) { Stop-Process -Id $bench.Id; $bench.WaitForExit() }
 throw
} finally {
 nvidia-smi | Set-Content "$out/nvidia-after.txt" -Encoding utf8
}
