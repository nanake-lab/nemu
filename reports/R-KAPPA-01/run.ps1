param([Parameter(Mandatory=$true)][ValidatePattern('^[a-zA-Z0-9_-]+$')][string]$RunName)
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$runRoot = Join-Path $PSScriptRoot ('evidence/' + $RunName)
if (Test-Path -LiteralPath $runRoot) { throw 'Evidence output already exists; choose a new RunName.' }
$active = Get-Process cargo,rustc,Runner.Worker -ErrorAction SilentlyContinue
if ($active) { throw ('Heavy workload active: ' + ($active.Name -join ', ')) }
New-Item -ItemType Directory -Path $runRoot | Out-Null
$env:CARGO_HOME = 'D:\Dev\cache\cargo'
$env:CARGO_TARGET_DIR = 'D:\Dev\cache\R-KAPPA-01\target'
$env:CARGO_BUILD_JOBS = '2'
$env:TEMP = 'D:\Dev\cache\R-KAPPA-01\tmp'
$env:TMP = $env:TEMP
New-Item -ItemType Directory -Path $env:TEMP -Force | Out-Null
$manifest = Join-Path $PSScriptRoot 'probe/Cargo.toml'

function Invoke-CargoCheck([string]$Name, [string[]]$CargoArgs) {
    $start = [DateTimeOffset]::UtcNow
    $timer = [Diagnostics.Stopwatch]::StartNew()
    & cargo +1.99.0 @CargoArgs *> (Join-Path $runRoot ($Name + '.log'))
    $code = $LASTEXITCODE
    $timer.Stop()
    [ordered]@{ name=$Name; command=@('cargo','+1.99.0') + $CargoArgs; cwd=$repoRoot;
        start_utc=$start.ToString('o'); end_utc=[DateTimeOffset]::UtcNow.ToString('o');
        exit_code=$code; wall_seconds=$timer.Elapsed.TotalSeconds } |
        ConvertTo-Json -Compress -Depth 5 | Add-Content -LiteralPath (Join-Path $runRoot 'commands.jsonl') -Encoding utf8
    Write-Output ($Name + ': exit ' + $code + ', ' + [math]::Round($timer.Elapsed.TotalSeconds,3) + ' s')
    if ($code -ne 0) { throw ($Name + ' failed; original log retained.') }
}

Push-Location $repoRoot
try {
    $files = @('AGENTS.md','core/Cargo.lock','core/rust-toolchain.toml',
        'core/crates/core-kernel/src/fast/drive.rs','core/crates/core-kernel/src/fast/space.rs',
        'core/crates/core-kernel/src/fast/types.rs','core/crates/core-runtime/src/lib.rs')
    $files += @(Get-ChildItem -LiteralPath (Join-Path $PSScriptRoot 'probe') -Recurse -File |
        Where-Object { $_.Extension -in '.rs','.toml' } | ForEach-Object { [IO.Path]::GetRelativePath($repoRoot,$_.FullName) })
    $hashes = @($files | ForEach-Object { $p=Join-Path $repoRoot $_; [ordered]@{path=$_;sha256=(Get-FileHash -LiteralPath $p -Algorithm SHA256).Hash} })
    [ordered]@{ head=(& git rev-parse HEAD); main_baseline=(& git rev-parse origin/main);
        rustc=(& rustc +1.99.0 -Vv); files=$hashes } |
        ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $runRoot 'tested-inputs.json') -Encoding utf8
    Invoke-CargoCheck 'build' @('build','--manifest-path',$manifest,'--offline')
    Invoke-CargoCheck 'tests' @('test','--manifest-path',$manifest,'--locked','--offline','--','--test-threads=1','--nocapture')
    Invoke-CargoCheck 'fmt' @('fmt','--manifest-path',$manifest,'--','--check')
    Invoke-CargoCheck 'clippy' @('clippy','--manifest-path',$manifest,'--all-targets','--locked','--offline','--','-D','warnings')
    $exe = Join-Path $env:CARGO_TARGET_DIR 'debug/r-kappa-01.exe'
    $trajectoryRoot = Join-Path $runRoot 'trajectories'
    $start = [DateTimeOffset]::UtcNow
    $timer = [Diagnostics.Stopwatch]::StartNew()
    $process = Start-Process -FilePath $exe -ArgumentList ('"' + $trajectoryRoot + '"') -WindowStyle Hidden -PassThru -Wait `
        -RedirectStandardOutput (Join-Path $runRoot 'experiment.stdout.log') -RedirectStandardError (Join-Path $runRoot 'experiment.stderr.log')
    $timer.Stop()
    [ordered]@{ command=@($exe,$trajectoryRoot); start_utc=$start.ToString('o');
        exit_code=$process.ExitCode; wall_seconds=$timer.Elapsed.TotalSeconds;
        cpu_seconds=$process.TotalProcessorTime.TotalSeconds; peak_working_set_bytes=$process.PeakWorkingSet64;
        exe_sha256=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash } |
        ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $runRoot 'experiment-process.json') -Encoding utf8
    if ($process.ExitCode -ne 0) { throw 'Experiment failed; evidence retained.' }
    Get-ChildItem -LiteralPath $runRoot -Recurse -File | ForEach-Object {
        [ordered]@{path=[IO.Path]::GetRelativePath($runRoot,$_.FullName);sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash;bytes=$_.Length}
    } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $runRoot 'evidence-hashes.json') -Encoding utf8
    Write-Output ('Evidence: ' + $runRoot)
} finally { Pop-Location }
