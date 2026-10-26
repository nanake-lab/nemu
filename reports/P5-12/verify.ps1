param(
    [Parameter(Mandatory=$true)]
    [ValidateSet('kernel', 'clippy', 'workspace', 'fmt')]
    [string]$Suite
)
$ErrorActionPreference = 'Stop'
$root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$cache = 'D:\Dev\cache\P5-12'
$env:CARGO_HOME = 'D:\Dev\cache\cargo'
$env:CARGO_TARGET_DIR = Join-Path $cache 'target'
$env:CARGO_BUILD_JOBS = '2'
$env:RUST_TEST_THREADS = '1'
$env:TEMP = Join-Path $cache 'tmp'
$env:TMP = $env:TEMP
New-Item -ItemType Directory -Force -Path $env:TEMP | Out-Null
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss-fff'
$evidence = Join-Path $PSScriptRoot 'verification'
New-Item -ItemType Directory -Force -Path $evidence | Out-Null
$prefix = Join-Path $evidence "$Suite-$stamp"
$busy = @(Get-Process -Name cargo,rustc,Runner.Worker,blender,electron,pytest -ErrorAction SilentlyContinue | Select-Object Name,Id)
if ($busy.Count -gt 0) {
    @{ status='not_run'; reason='other workload'; processes=$busy } | ConvertTo-Json -Depth 5 | Set-Content "$prefix.json" -Encoding utf8
    Write-Output 'Another heavy workload is active; no test started.'
    exit 75
}
$cargoArgs = switch ($Suite) {
    'kernel' { @('test','-p','core-kernel','--locked','--offline') }
    'clippy' { @('clippy','--workspace','--all-targets','--locked','--offline','--','-D','warnings') }
    'workspace' { @('test','--workspace','--locked','--offline') }
    'fmt' { @('fmt','--all','--','--check') }
}
Push-Location (Join-Path $root 'core')
try {
    $started = (Get-Date).ToUniversalTime().ToString('o')
    $revision = git rev-parse HEAD
    $rust = rustc --version
    $inputs = @(Get-ChildItem -LiteralPath (Join-Path $root 'core/crates/core-kernel/src') -Recurse -File | ForEach-Object {
        @{path=[System.IO.Path]::GetRelativePath($root,$_.FullName);sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}
    })
    & cargo @cargoArgs 2>&1 | Tee-Object -FilePath "$prefix.log"
    $code = $LASTEXITCODE
    @{ command=('cargo '+($cargoArgs -join ' ')); cwd=(Get-Location).Path; started_utc=$started; ended_utc=(Get-Date).ToUniversalTime().ToString('o'); exit_code=$code; revision=$revision; rust=$rust; cargo_home=$env:CARGO_HOME; target=$env:CARGO_TARGET_DIR; temp=$env:TEMP; jobs=2; test_threads=1; inputs=$inputs } | ConvertTo-Json -Depth 5 | Set-Content "$prefix.json" -Encoding utf8
    Write-Output "Evidence: $prefix; exit: $code"
} finally { Pop-Location }
exit $code
