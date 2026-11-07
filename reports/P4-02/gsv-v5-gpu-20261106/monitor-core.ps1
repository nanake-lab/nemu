# Pure classification is shared by the no-GPU fixture and live monitor.
function Get-OwnedIds {
 param([int]$ActualId,[object[]]$Snapshot)
 $ids=[System.Collections.Generic.HashSet[int]]::new()
 [void]$ids.Add($ActualId)
 do {
  $added=$false
  foreach($item in $Snapshot) {
   if($ids.Contains([int]$item.ParentProcessId) -and $ids.Add([int]$item.ProcessId)) { $added=$true }
  }
 } while($added)
 return ,$ids
}
function Confirm-ParentChain {
 param([int]$ActualId,[int]$LauncherId,[object[]]$Snapshot)
 $chain=[System.Collections.Generic.List[int]]::new()
 $cursor=$ActualId
 while(!$chain.Contains($cursor)) {
  $chain.Add($cursor)
  if($cursor -eq $LauncherId) { return ,$chain.ToArray() }
  $row=@($Snapshot | Where-Object { [int]$_.ProcessId -eq $cursor })
  if($row.Count -ne 1) { break }
  $cursor=[int]$row[0].ParentProcessId
 }
 throw 'imports_ready PID is not the launcher or its verified descendant'
}
function Get-ExternalHeavyProcesses {
 param([object[]]$Snapshot,[System.Collections.Generic.HashSet[int]]$Owned)
 @($Snapshot | Where-Object {
  !$Owned.Contains([int]$_.ProcessId) -and (
   $_.Name -match '^(Runner\.(Listener|Worker)|llama-server|llama-cli|ollama|blender|vlc|mpv|ffplay|GenshinImpact|StarRail|ZenlessZoneZero|League of Legends|VALORANT.*|cs2|dota2)\.exe$' -or
   ($_.Name -match '^python' -and $_.CommandLine -match 'llama|sovits|comfyui|stable.diffusion|vllm|text-generation|inference')
  )
 })
}
function Get-ExternalHeavyEngines {
 param([object[]]$Engines,[System.Collections.Generic.HashSet[int]]$Owned)
 @($Engines | Where-Object {
  $match=[regex]::Match($_.Name,'^pid_(\d+)_')
  $match.Success -and !$Owned.Contains([int]$match.Groups[1].Value) -and
  $_.UtilizationPercentage -gt 0 -and $_.Name -match 'VideoDecode|VideoProcessing|engtype_Compute|engtype_Cuda'
 })
}
function Get-OwnedMemory {
 param([System.Collections.Generic.HashSet[int]]$Owned)
 @($Owned | ForEach-Object {
  $process=Get-Process -Id $_ -ErrorAction SilentlyContinue
  if($process) { [pscustomobject]@{pid=$process.Id;rss=$process.WorkingSet64;private=$process.PrivateMemorySize64;start_utc=$process.StartTime.ToUniversalTime().ToString('o')} }
 })
}
function Wait-ReadyEvent {
 param([string]$Path,[System.Diagnostics.Process]$Launcher)
 # 120 seconds permits cold Python imports; no model/CUDA work before ready handshake.
 $until=[DateTime]::UtcNow.AddSeconds(120)
 while([DateTime]::UtcNow -lt $until) {
  if(Test-Path $Path) {
   $content=[System.IO.File]::ReadAllText($Path)
   $lines=$content.Split("`n")
   for($i=0;$i -lt $lines.Length-1;$i++) {
    $row=$lines[$i] | ConvertFrom-Json
    if($row.event -eq 'imports_ready') { return $row }
   }
  }
  $Launcher.Refresh()
  if($Launcher.HasExited) { throw 'Python exited before imports_ready' }
  Start-Sleep -Milliseconds 100
 }
 throw 'imports_ready handshake timed out'
}
