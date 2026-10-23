$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
Set-Location D:\Dev\nemu-wt\P4-02
# User requested that all running work finish naturally. Do not stop any process.
while (@(Get-CimInstance Win32_Process -Filter "Name='pwsh.exe'" | Where-Object {$_.CommandLine -match '-File D:\\Dev\\cache\\p4-02\\(?:isolation-retest|post-merge-cpu-probe)-20261023\.ps1\s*$'}).Count) { Start-Sleep -Seconds 5 }
while (@(Get-Process -Name 'Runner.Worker','cargo','rustc','MSBuild','blender' -ErrorAction SilentlyContinue).Count) { Start-Sleep -Seconds 5 }
$env:TEMP='D:\Dev\cache\tmp'; $env:TMP=$env:TEMP
$env:CARGO_HOME='D:\Dev\cache\cargo'
$env:CARGO_TARGET_DIR='D:\Dev\cache\p4-02\cargo-private-20261023'
New-Item -ItemType Directory -Path $env:CARGO_TARGET_DIR -Force | Out-Null
$link=[IO.Path]::GetFullPath('D:\Dev\nemu-wt\P4-02\core\target')
$existing=Get-Item -LiteralPath $link
if ($link -ne 'D:\Dev\nemu-wt\P4-02\core\target' -or $existing.LinkType -ne 'Junction' -or $existing.Target -ne 'D:\Dev\cache\p4-02\cargo-target') { throw 'Unexpected local target link' }
# Remove only the local junction, never the existing shared cache directory.
Remove-Item -LiteralPath $link -Force
if (!(Test-Path -LiteralPath 'D:\Dev\cache\p4-02\cargo-target\debug\deps')) { throw 'Original cache missing' }
New-Item -ItemType Junction -Path $link -Target $env:CARGO_TARGET_DIR | Out-Null
$start=[DateTime]::UtcNow.ToString('o')
Push-Location core
try {
  cargo test -p core-store --doc --locked 2>&1 | Tee-Object D:\Dev\cache\p4-02\storage-doc-private-target-20261023.log
  $code=$LASTEXITCODE
} finally { Pop-Location }
@{startUtc=$start;endUtc=[DateTime]::UtcNow.ToString('o');exitCode=$code;target=$env:CARGO_TARGET_DIR;userVariableAbsent= !((Get-Item HKCU:\Environment).GetValueNames() -contains 'CARGO_TARGET_DIR');originalCachePreserved=(Test-Path -LiteralPath 'D:\Dev\cache\p4-02\cargo-target\debug\deps');note='31 runtime tests already passed; recheck failed rustdoc part in fresh process-local target. Original full-command failure retained.'} | ConvertTo-Json | Set-Content D:\Dev\cache\p4-02\storage-doc-private-target-20261023.json -Encoding utf8
exit $code
