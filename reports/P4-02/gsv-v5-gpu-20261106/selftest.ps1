$ErrorActionPreference='Stop'
. "$PSScriptRoot/monitor-core.ps1"
$out="$PSScriptRoot/selftest"
$python='D:\Dev\cache\p4-02\gsv-v5-venv\Scripts\python.exe'
$launch=$null; $external=$null
try {
 $launch=Start-Process $python -ArgumentList @('-u',"$PSScriptRoot/fixture.py",$out,'root') -WindowStyle Hidden -PassThru
 $external=Start-Process $python -ArgumentList @('-u',"$PSScriptRoot/fixture.py",$out,'external-inference') -WindowStyle Hidden -PassThru
 $ready=Wait-ReadyEvent "$out/events.jsonl" $launch
 $until=[DateTime]::UtcNow.AddSeconds(20)
 while(!(Test-Path "$out/child.json") -or !(Test-Path "$out/external-inference.json")) {
  if([DateTime]::UtcNow -gt $until) { throw 'fixture child handshake timed out' }
  Start-Sleep -Milliseconds 100
 }
 $child=Get-Content "$out/child.json" -Raw | ConvertFrom-Json
 $ext=Get-Content "$out/external-inference.json" -Raw | ConvertFrom-Json
 $snapshot=@(Get-CimInstance Win32_Process)
 $chain=Confirm-ParentChain $ready.pid $launch.Id $snapshot
 $owned=Get-OwnedIds $ready.pid $snapshot
 if(!$owned.Contains([int]$child.pid) -or $owned.Contains([int]$ext.pid)) { throw 'Wrong ownership tree' }
 # Fixture engines are synthetic; process tree and memory are real OS observations.
 $engines=@(
  [pscustomobject]@{Name="pid_$($ready.pid)_engtype_Cuda";UtilizationPercentage=20},
  [pscustomobject]@{Name="pid_$($child.pid)_engtype_Compute";UtilizationPercentage=10},
  [pscustomobject]@{Name="pid_$($ext.pid)_engtype_Cuda";UtilizationPercentage=15}
 )
 $found=@(Get-ExternalHeavyEngines $engines $owned)
 if($found.Count -ne 1 -or $found[0].Name -ne "pid_$($ext.pid)_engtype_Cuda") { throw 'Own tree exclusion/external engine classification failed' }
 $heavy=@(Get-ExternalHeavyProcesses $snapshot $owned)
 if(!(@($heavy.ProcessId) -contains $ext.pid) -or @($heavy | Where-Object {$owned.Contains([int]$_.ProcessId)}).Count) { throw 'External process detection failed' }
 $memory=@(Get-OwnedMemory $owned)
 $actual=@($memory | Where-Object pid -eq $ready.pid)
 if($actual.Count -ne 1 -or $actual[0].rss -lt 64MB -or $actual[0].private -lt 64MB) { throw 'Actual Python memory not sampled' }
 if($ready.pid -ne $launch.Id -and @($memory | Where-Object pid -eq $launch.Id).Count) { throw 'Launcher incorrectly sampled' }
 [ordered]@{passed=$true;gpu_imports=$false;launcher=$launch.Id;actual=$ready.pid;parent_chain=$chain;owned=@($owned);child=$child;external=$ext;memory=$memory;external_engine=$found;external_process_ids=@($heavy.ProcessId);checks=@('own root and descendant excluded','external process and synthetic engine detected','RSS/private sampled on real Python allocation')} | ConvertTo-Json -Depth 7 | Set-Content "$out/result.json" -Encoding utf8
 Get-Content "$out/result.json"
} catch {
 $_ | Out-String | Set-Content "$out/failure.txt"
 throw
} finally {
 'finish' | Set-Content "$out/finish"
 if($launch) { $launch.WaitForExit(10000) | Out-Null }
 if($external) { $external.WaitForExit(10000) | Out-Null }
}
