# Guard the self-hosted runner selection without starting Windows jobs.
$ErrorActionPreference = 'Stop'
$scriptPath = Join-Path $PSScriptRoot 'windows-ci-scope.ps1'
$env:GITHUB_OUTPUT = [IO.Path]::GetTempFileName()
$env:PR_BASE = 'base'
$env:PR_HEAD = 'head'
$env:BEFORE = 'before'
$env:DEFAULT_REF = 'refs/remotes/origin/main'

function git {
    if (($args -join ' ') -ne "-c core.quotepath=false diff --name-only --no-renames $comparison") {
        throw "Unexpected comparison: $args"
    }
    $global:LASTEXITCODE = $diffExit
    $changedPathsFixture
}

function Assert-Scope {
    param(
        [string]$Name, [string[]]$Files, [string]$Expected,
        [string]$Event = 'pull_request', [string]$Comparison = 'base...head',
        [string]$Full = '', [int]$DiffExit = 0
    )
    $changedPathsFixture = $Files
    $env:EVENT = $Event
    $env:FULL_INPUT = $Full
    Clear-Content -LiteralPath $env:GITHUB_OUTPUT
    & $scriptPath | Out-Null
    $actual = (Get-Content -LiteralPath $env:GITHUB_OUTPUT) -join ','
    if ($actual -ne $Expected) { throw "$Name expected $Expected but got $actual" }
    Write-Output "PASS $Name"
}

$none = 'desktop=false,gateway=false,selected=false,storage=false'
$desktop = 'desktop=true,gateway=false,selected=true,storage=false'
$native = 'desktop=false,gateway=true,selected=true,storage=true'
$all = 'desktop=true,gateway=true,selected=true,storage=true'
try {
    Assert-Scope 'pure metadata' @('apps/stage/manifest.json', 'packages/body/说明.md', 'packages/body/LICENSE-MIT') $none
    Assert-Scope 'pure Markdown' @('apps/stage/README.md', 'packages/body/docs/guide.md') $none
    Assert-Scope 'manifest and desktop source' @('apps/stage/manifest.json', 'apps/stage/src/index.ts') $desktop
    Assert-Scope 'licence and primary storage' @('packages/body/LICENCE.txt', 'core/crates/core-store/src/store.rs') $all
    Assert-Scope 'storage only' @('core/crates/core-store/src/store.rs') $native
    Assert-Scope 'gateway only' @('core/crates/gateway/src/gateway.rs') 'desktop=false,gateway=true,selected=true,storage=false'
    Assert-Scope 'workflow dispatch uses full branch diff' @('apps/stage/manifest.json', 'apps/stage/src/index.ts') $desktop -Event workflow_dispatch -Comparison 'refs/remotes/origin/main...HEAD'
    Assert-Scope 'explicit full overrides paths' @() $all -Event workflow_dispatch -Full true -Comparison 'unexpected comparison'
    Assert-Scope 'push compares before to HEAD' @('apps/stage/manifest.json') $none -Event push -Comparison 'before HEAD'
    Assert-Scope 'unavailable comparison conservatively selects all' @() $all -DiffExit 128
    Assert-Scope 'unrecognizable escaped path conservatively selects all' @('"apps/strange\nfile.md"') $all
    Assert-Scope 'empty scope' @() $none
} finally {
    Remove-Item -LiteralPath $env:GITHUB_OUTPUT
}
