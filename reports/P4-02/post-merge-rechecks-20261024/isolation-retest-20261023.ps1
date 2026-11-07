$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
Set-Location D:\Dev\nemu-wt\P4-02
$evidenceBase='D:\Dev\cache\p4-02\isolation-retest-20261023-run1'
New-Item -ItemType Directory -Path $evidenceBase -Force | Out-Null
$oldTemp='D:\Dev\cache\tmp'
$defaultTemp=Join-Path $env:LOCALAPPDATA 'Temp'
$before=@{sharedTemp=(Get-Acl -LiteralPath $oldTemp).Sddl;defaultTemp=(Get-Acl -LiteralPath $defaultTemp).Sddl}
$before | ConvertTo-Json | Set-Content (Join-Path $evidenceBase 'before-acls.json') -Encoding utf8
# Keep the CPU probe and its wheel build ahead of this .NET smoke.
while (@(Get-CimInstance Win32_Process -Filter "Name='pwsh.exe'" | Where-Object {$_.CommandLine -match '-File D:\\Dev\\cache\\p4-02\\post-merge-cpu-probe-20261023\.ps1\s*$'}).Count) { Start-Sleep -Seconds 5 }
while (@(Get-Process -Name 'Runner.Worker','cargo','rustc','MSBuild','blender' -ErrorAction SilentlyContinue).Count) { Start-Sleep -Seconds 5 }
$env:TEMP='D:\Dev\cache\p4-02\isolation-temp'
$env:TMP=$env:TEMP
$env:DOTNET_CLI_HOME='D:\Dev\cache\p4-02\dotnet-home'
$env:NUGET_PACKAGES='D:\Dev\cache\p4-02\nuget'
$env:DOTNET_CLI_TELEMETRY_OPTOUT='1'
$env:GITHUB_RUN_ID='local-p4-02-isolation-20261023'
$env:GITHUB_RUN_ATTEMPT='1'
$env:RUNNER_TEMP=$evidenceBase
$env:NEMU_EVIDENCE_DIR=Join-Path $evidenceBase 'nemu-evidence/local-p4-02-isolation-20261023/1/windows'
$start=[DateTime]::UtcNow.ToString('o')
pwsh -NoProfile -File tools/scaffold/windows-evidence.ps1 -Mode Init 2>&1 | Tee-Object (Join-Path $evidenceBase 'retest.log')
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
pwsh -NoProfile -File tools/scaffold/windows-evidence.ps1 -Mode Run -Check isolation 2>&1 | Tee-Object (Join-Path $evidenceBase 'retest.log') -Append
$code=$LASTEXITCODE
pwsh -NoProfile -File tools/scaffold/windows-evidence.ps1 -Mode Verify 2>&1 | Tee-Object (Join-Path $evidenceBase 'retest.log') -Append
$verify=$LASTEXITCODE
$after=@{sharedTemp=(Get-Acl -LiteralPath $oldTemp).Sddl;defaultTemp=(Get-Acl -LiteralPath $defaultTemp).Sddl}
$after | ConvertTo-Json | Set-Content (Join-Path $evidenceBase 'after-acls.json') -Encoding utf8
$unchanged=$before.sharedTemp -eq $after.sharedTemp -and $before.defaultTemp -eq $after.defaultTemp
@{startedUtc=$start;endedUtc=[DateTime]::UtcNow.ToString('o');exitCode=$code;provenanceExit=$verify;existingAclsUnchanged=$unchanged;temporaryEnvironmentOnly=$true;temp=$env:TEMP} | ConvertTo-Json | Set-Content (Join-Path $evidenceBase 'result.json') -Encoding utf8
if (!$unchanged) { throw 'Existing ACL changed unexpectedly' }
exit $code

