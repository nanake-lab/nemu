$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
Set-Location D:\Dev\nemu-wt\P4-02
$env:TEMP='D:\Dev\cache\tmp'; $env:TMP=$env:TEMP
$env:UV_CACHE_DIR='D:\Dev\cache\uv'
$env:UV_PROJECT_ENVIRONMENT='D:\Dev\nemu-wt\P4-02\services\voice-sidecar\.venv'
while ($true) {
  $results=Get-Content -LiteralPath D:\Dev\cache\p4-02\merge-main-checks-20261023\results.json -Raw | ConvertFrom-Json
  if (@($results | Where-Object {$_.name -eq 'storage-serial'}).Count -eq 1) { break }
  Start-Sleep -Seconds 5
}
while (@(Get-Process -Name 'Runner.Worker','cargo','rustc','MSBuild','blender' -ErrorAction SilentlyContinue).Count) { Start-Sleep -Seconds 5 }
Write-Output 'START CPU transport resource probe (no model/GPU)'
& services\voice-sidecar\.venv\Scripts\python.exe D:\Dev\cache\p4-02\transport-resource-probe-20261023.py 2>&1 | Tee-Object D:\Dev\cache\p4-02\transport-resource-probe-20261023.log
$code=$LASTEXITCODE
Write-Output "CPU probe exit=$code"
if ($code -ne 0) { exit $code }
uv build --project services/voice-sidecar 2>&1 | Tee-Object D:\Dev\cache\p4-02\rss-sampler-build-20261023.log
exit $LASTEXITCODE
