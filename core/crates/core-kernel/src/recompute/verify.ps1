param(
    [ValidateSet('Test','Clippy','WorkspaceTest','WorkspaceClippy','Format','WorkspaceFormat')][string]$Mode = 'Test',
    [Parameter(Mandatory=$true)][string]$RunId
)
$ErrorActionPreference = 'Stop'
if ($RunId -notmatch '^[a-zA-Z0-9-]+$') { throw 'Invalid run ID' }
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../../..')).Path
$cache = 'D:\Dev\cache\P5-26'
$harness = Join-Path $cache 'harness'
$evidence = Join-Path $root "reports/P5-26/verification/$RunId"
if (Test-Path -LiteralPath $evidence) { throw 'Evidence directory already exists' }
New-Item -ItemType Directory -Path $harness,$evidence -Force | Out-Null
$kernelPath = (Join-Path $root 'core/crates/core-kernel').Replace('\','/')
$typesPath = (Join-Path $root 'core/crates/core-types').Replace('\','/')
$modulePath = (Join-Path $PSScriptRoot 'mod.rs').Replace('\','/')
$evidenceModule = (Join-Path $PSScriptRoot '../evidence/mod.rs').Replace('\','/')
@"
[package]
name = "p5-26-recompute-verifier"
version = "0.0.0"
edition = "2024"
[workspace]
[lib]
path = "harness.rs"
[dependencies]
core-kernel = { path = "$kernelPath" }
core-types = { path = "$typesPath" }
[dev-dependencies]
serde_json = "=1.0.154"
"@ | Set-Content -LiteralPath (Join-Path $harness 'Cargo.toml') -Encoding utf8NoBOM
@"
#![forbid(unsafe_code)]
pub use core_kernel::{law, dynamics, modulation, effects, fast};
#[path = "$evidenceModule"]
pub mod evidence;
#[path = "$modulePath"]
pub mod recompute;
"@ | Set-Content -LiteralPath (Join-Path $harness 'harness.rs') -Encoding utf8NoBOM
$env:CARGO_TARGET_DIR = Join-Path $cache $(if ($Mode.StartsWith('Workspace')) { 'target' } else { 'target-harness' })
$env:CARGO_HOME = 'D:\Dev\cache\cargo'
$env:CARGO_BUILD_JOBS = '4'
$env:TEMP = Join-Path $cache 'tmp'
$env:TMP = $env:TEMP
New-Item -ItemType Directory -Path $env:TEMP -Force | Out-Null
$command = if ($Mode -eq 'Format') { 'rustfmt' } else { 'cargo' }
$arguments = switch ($Mode) {
    'Test' { @('test','--offline','--manifest-path',(Join-Path $harness 'Cargo.toml'),'--','--nocapture','--test-threads=1') }
    'Clippy' { @('clippy','--offline','--locked','--manifest-path',(Join-Path $harness 'Cargo.toml'),'--all-targets','--','-D','warnings') }
    'WorkspaceTest' { @('test','--workspace','--locked','--offline','--','--nocapture','--test-threads=1') }
    'WorkspaceClippy' { @('clippy','--workspace','--all-targets','--locked','--offline','--','-D','warnings') }
    'Format' { @('--edition','2024','--check',$modulePath) }
    'WorkspaceFormat' { @('fmt','--all','--','--check') }
}
$sources = Get-ChildItem -LiteralPath $PSScriptRoot -Recurse -File | ForEach-Object {
    @{ path = $_.FullName.Substring($root.Length + 1).Replace('\','/'); sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash }
}
$sources | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $evidence 'source-sha256.json') -Encoding utf8NoBOM
$startHead = (& git -C $root rev-parse HEAD)
$startBaseline = (& git -C $root rev-parse origin/main)
$start = [DateTimeOffset]::UtcNow
Push-Location (Join-Path $root 'core')
try {
    & $command @arguments 1> (Join-Path $evidence 'stdout.log') 2> (Join-Path $evidence 'stderr.log')
    $code = $LASTEXITCODE
    $lockPath = if ($Mode.StartsWith('Workspace')) { Join-Path $root 'core/Cargo.lock' } else { Join-Path $harness 'Cargo.lock' }
    if (Test-Path -LiteralPath $lockPath) {
        Copy-Item -LiteralPath $lockPath -Destination (Join-Path $evidence 'Cargo.lock')
    }
    @{ command = $command; arguments = $arguments; cwd = (Get-Location).Path;
       started_utc = $start.ToString('o'); ended_utc = [DateTimeOffset]::UtcNow.ToString('o');
       exit_code = $code; baseline = $startBaseline; head = $startHead; ended_head = (& git rev-parse HEAD);
       rustc = (& rustc --version); target = $env:CARGO_TARGET_DIR } | ConvertTo-Json -Depth 4 |
       Set-Content -LiteralPath (Join-Path $evidence 'result.json') -Encoding utf8NoBOM
} finally { Pop-Location }
Get-Content -LiteralPath (Join-Path $evidence 'stdout.log') -Tail 12
Get-Content -LiteralPath (Join-Path $evidence 'stderr.log') -Tail 40
exit $code
