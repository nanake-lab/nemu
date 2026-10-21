[CmdletBinding()]
param(
    [string]$Config = (Join-Path $PSScriptRoot '../../content/nemu/build.yaml'),
    [string]$Output = (Join-Path $PSScriptRoot '.cache/build'),
    [string]$Step,
    [string]$InputBlend,
    [string]$OutputBlend
)
$ErrorActionPreference = 'Stop'
$busy = @(Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.ProcessName -match '^(blender|electron|Runner\.Worker)$' })
if ($busy.Count -gt 0) { throw "step=schedule object=processes expected=exclusive Blender window actual=$($busy.ProcessName -join ',')" }
$python = if ($env:NEMU_ASSET_PYTHON) { $env:NEMU_ASSET_PYTHON } else { Join-Path $PSScriptRoot '.venv/Scripts/python.exe' }
if (-not (Test-Path -LiteralPath $python -PathType Leaf)) { throw "Asset Python missing: $python. Prepare the pinned environment using the offline commands in tools/asset-pipeline/README.md." }
$arguments = @((Join-Path $PSScriptRoot 'pipeline.py'), '--config', $Config, '--output', $Output)
if ($Step) { $arguments += @('--step', $Step, $InputBlend, $OutputBlend) }
& $python @arguments
exit $LASTEXITCODE
