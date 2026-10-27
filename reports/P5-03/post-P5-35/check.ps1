$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$env:CARGO_HOME = 'D:\Dev\cache\cargo'
$env:CARGO_TARGET_DIR = 'D:\Dev\cache\cargo-target\P5-03'
$env:UV_CACHE_DIR = 'D:\Dev\cache\uv'
$env:npm_config_cache = 'D:\Dev\cache\npm'
$env:PNPM_STORE_DIR = 'D:\Dev\cache\pnpm'
$env:HF_HOME = 'D:\Dev\cache\huggingface'
$env:TEMP = 'D:\Dev\cache\P5-03\tmp'
$env:TMP = $env:TEMP
New-Item -ItemType Directory -Force $env:TEMP | Out-Null
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$run = Join-Path $PSScriptRoot ('runs/' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $run | Out-Null
$env:NEMU_W18_EVIDENCE_DIR = Join-Path $run 'SYS01'
$results = @()
function Wait-HeavySlot {
    $busy = @(Get-Process cargo,rustc,Runner.Worker,blender -ErrorAction SilentlyContinue)
    if ($busy.Count) { 'Waiting for the existing local heavy load (no process is stopped).' }
    while ($busy.Count) {
        Start-Sleep -Seconds 5
        $busy = @(Get-Process cargo,rustc,Runner.Worker,blender -ErrorAction SilentlyContinue)
    }
}
Push-Location $repo
try {
    $files = @(git ls-files --cached --others --exclude-standard -- core/crates/gateway core/Cargo.lock .github/workflows/windows.yml)
    $manifest = foreach ($file in $files) {
        # git hash-object applies the repository's normal clean/line-ending rules.
        [pscustomobject]@{ path = $file; git_blob = (git hash-object -- $file) }
    }
    [ordered]@{
        commit = (git rev-parse HEAD)
        main = (git rev-parse origin/main)
        dirty = @(git status --porcelain)
        captured = (Get-Date -Format o)
        files = $manifest
    } | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $run 'source-manifest.json') -Encoding utf8
    cargo -Vv | Set-Content (Join-Path $run 'toolchain.log') -Encoding utf8
    rustc -Vv | Add-Content (Join-Path $run 'toolchain.log') -Encoding utf8
    Set-Location (Join-Path $repo 'core')
    $commands = @(
        @{ name='fmt'; args=@('fmt','--all','--','--check') },
        @{ name='clippy'; args=@('clippy','--workspace','--all-targets','--locked','--','-D','warnings') },
        @{ name='gateway'; args=@('test','-p','gateway','--locked','--','--test-threads=1','--nocapture') }
    )
    foreach ($command in $commands) {
        Wait-HeavySlot
        $start = Get-Date
        $ErrorActionPreference = 'Continue'
        $cargoArgs = $command.args
        & cargo @cargoArgs 2>&1 | Tee-Object -FilePath (Join-Path $run ($command.name + '.log'))
        $code = $LASTEXITCODE
        $ErrorActionPreference = 'Stop'
        $results += [pscustomobject]@{ command=('cargo ' + ($command.args -join ' ')); exit_code=$code; started=$start.ToString('o'); elapsed_seconds=((Get-Date)-$start).TotalSeconds }
        if ($code -ne 0) { throw ($command.name + ' failed; raw output retained in ' + $run) }
    }
} finally {
    $results | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $run 'result.json') -Encoding utf8
    Pop-Location
    Write-Output ('Evidence: ' + $run)
}
