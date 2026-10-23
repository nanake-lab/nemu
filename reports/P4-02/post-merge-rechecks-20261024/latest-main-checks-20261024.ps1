$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
Set-Location D:\Dev\nemu-wt\P4-02
while (@(Get-CimInstance Win32_Process -Filter "Name='pwsh.exe'" | Where-Object {$_.CommandLine -match '-File D:\\Dev\\cache\\p4-02\\(?:post-merge-cpu-probe|isolation-retest|storage-doc-private-target)-20261023\.ps1\s*$'}).Count) { Start-Sleep -Seconds 5 }
$env:UV_CACHE_DIR='D:\Dev\cache\uv'
$env:UV_PYTHON_INSTALL_DIR='D:\Dev\cache\uv-python'
$env:npm_config_cache='D:\Dev\cache\npm'
$env:PNPM_STORE_DIR='D:\Dev\cache\pnpm'
$env:HF_HOME='D:\Dev\cache\huggingface'
$env:TORCH_HOME='D:\Dev\cache\torch'
$env:TORCHINDUCTOR_CACHE_DIR='D:\Dev\cache\torchinductor'
$env:TRITON_CACHE_DIR='D:\Dev\cache\triton'
$env:CARGO_HOME='D:\Dev\cache\cargo'
$env:CARGO_TARGET_DIR='D:\Dev\cache\p4-02\cargo-private-20261023'
$env:DOTNET_CLI_HOME='D:\Dev\cache\p4-02\dotnet-home'
$env:NUGET_PACKAGES='D:\Dev\cache\p4-02\nuget'
$env:DOTNET_CLI_TELEMETRY_OPTOUT='1'
$env:PLAYWRIGHT_BROWSERS_PATH='D:\Dev\cache\playwright'
$env:NODE_COMPILE_CACHE='D:\Dev\cache\node-compile'
$env:ELECTRON_CACHE='D:\Dev\cache\electron'
$env:TEMP='D:\Dev\cache\tmp'
$env:TMP=$env:TEMP
$env:UV_LINK_MODE='copy'
$pythonHome='D:\Dev\cache\uv-python\cpython-3.12.14-windows-x86_64-none'
$env:PATH="$pythonHome;$env:PATH"
Remove-Item Env:NEMU_RUN_TTS_GPU_TESTS -ErrorAction SilentlyContinue
$script:out='D:\Dev\cache\p4-02\latest-main-checks-20261024'
New-Item -ItemType Directory -Path $script:out -Force | Out-Null
$script:results=[Collections.Generic.List[object]]::new()
function Gate {
  while (@(Get-Process -Name 'Runner.Worker','blender','cargo','rustc','MSBuild' -ErrorAction SilentlyContinue).Count) { Start-Sleep -Seconds 5 }
}
function Check([string]$name,[scriptblock]$body) {
  Gate
  $start=[DateTime]::UtcNow
  Write-Output "START $name $($start.ToString('o'))"
  $code=1
  try { & $body 2>&1 | Tee-Object -FilePath (Join-Path $script:out "$name.log"); $code=$LASTEXITCODE } catch { $_ | Out-String | Tee-Object -FilePath (Join-Path $script:out "$name.log") -Append; $code=1 }
  $script:results.Add([ordered]@{name=$name;startUtc=$start.ToString('o');endUtc=[DateTime]::UtcNow.ToString('o');exitCode=$code})
  $script:results | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $script:out 'results.json') -Encoding utf8
  Write-Output "END $name exit=$code"
}
Check 'node' {
  pnpm install --frozen-lockfile --store-dir D:/Dev/cache/pnpm
  if ($LASTEXITCODE -ne 0) { return }
  foreach ($command in @('lint','typecheck','test','build','test:contracts','test:transport','tts:schema','ci:budget','test:p205:metrics')) {
    pnpm $command
    if ($LASTEXITCODE -ne 0) { return }
  }
}
foreach ($project in @('workers/irl','workers/drive','workers/model-adapter','services/voice-sidecar')) {
  $projectName=$project.Replace('/','-')
  $env:UV_PROJECT_ENVIRONMENT=if ($project -eq 'services/voice-sidecar') {'D:\Dev\nemu-wt\P4-02\services\voice-sidecar\.venv'} else {Join-Path 'D:\Dev\cache\p4-02\check-envs' $projectName}
  Push-Location $project
  try { Check $projectName {
    uv sync --locked
    if ($LASTEXITCODE -ne 0) { return }
    uv run ruff check .
    if ($LASTEXITCODE -ne 0) { return }
    uv run mypy --strict src tests
    if ($LASTEXITCODE -ne 0) { return }
    uv run mypy --strict --platform linux src tests
    if ($LASTEXITCODE -ne 0) { return }
    uv run pytest
    if ($LASTEXITCODE -ne 0) { return }
    uv build
  } } finally { Pop-Location }
}
$env:GITHUB_RUN_ID='local-p4-02-latest-main-20261024'
$env:GITHUB_RUN_ATTEMPT='1'
$env:RUNNER_TEMP=$script:out
$env:NEMU_EVIDENCE_DIR=Join-Path $script:out 'nemu-evidence/local-p4-02-latest-main-20261024/1/windows'
Check 'windows-init' { pwsh -NoProfile -File tools/scaffold/windows-evidence.ps1 -Mode Init }
Check 'windows-e2e' { pwsh -NoProfile -File tools/scaffold/windows-evidence.ps1 -Mode Run -Check e2e }
Check 'windows-visual' { pwsh -NoProfile -File tools/scaffold/windows-evidence.ps1 -Mode Run -Check visual }
Check 'windows-verify' { pwsh -NoProfile -File tools/scaffold/windows-evidence.ps1 -Mode Verify }
$failed=@($script:results | Where-Object {$_.exitCode -ne 0}).Count
Write-Output "COMPLETE failed=$failed"
if ($failed) { exit 1 }
