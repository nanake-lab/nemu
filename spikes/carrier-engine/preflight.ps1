[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$OutputDirectory,
    [string]$EngineRoot = 'D:\Dev\cache\P0-10\llama.cpp',
    [string]$ModelRoot = 'D:\Dev\cache\P0-10\models'
)

# Inventory only: no library loading, model loading, GPU query, or installation.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
$outputRoot = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $outputRoot) {
    throw 'Use a new output directory; existing evidence must not be overwritten.'
}

$started = [DateTimeOffset]::UtcNow.ToString('o')
$runner = @(Get-Process | Where-Object {
    $_.ProcessName -in @('Runner.Listener', 'Runner.Worker')
} | Select-Object ProcessName, Id)
$commands = @(foreach ($name in @('llama-cli', 'llama-server', 'llama-bench', 'llama-quantize')) {
    $command = Get-Command -Name $name -CommandType Application -ErrorAction SilentlyContinue
    [pscustomobject]@{ name = $name; found = ($null -ne $command) }
})
$artifacts = @(foreach ($name in @('llama-cli.exe', 'llama-server.exe', 'llama-bench.exe', 'llama-quantize.exe', 'llama.dll')) {
    $path = Join-Path $EngineRoot $name
    [pscustomobject]@{ path = $path; exists = (Test-Path -LiteralPath $path -PathType Leaf) }
})
$models = @(foreach ($name in @('Q4_K_M.gguf', 'Q8_0.gguf')) {
    $path = Join-Path $ModelRoot $name
    [pscustomobject]@{ path = $path; exists = (Test-Path -LiteralPath $path -PathType Leaf) }
})
$inputPaths = @(
    'AGENTS.md', 'docs/ARCHITECTURE.md', 'docs/CONTRACTS.md',
    'docs/echo-core/README.md', 'tasks/P0-foundation.md',
    'reports/P0-09/DECISION.md', 'reports/P0-07/decision.md',
    'reports/P0-08/REVIEW-R1-REPORT.md',
    'docs/adr/0012-continuity-carrier-mobile.md',
    'docs/adr/0027-voice-model-breeze-target-gsv-interim.md',
    'workers/model-adapter/probe/injection.py',
    'spikes/codex-app-server/fixtures/irl-cases.json',
    'spikes/codex-app-server/semantic-candidate.schema.json',
    'spikes/codex-app-server/semantic-candidate.wire.schema.json',
    'spikes/codex-app-server/src/local_breakdown.py',
    'workers/model-adapter/probe/latency_probe.py'
)
$inputs = @(foreach ($relative in $inputPaths) {
    $path = Join-Path $repoRoot $relative
    [pscustomobject]@{
        path = $relative
        sha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    }
})
$fixtures = Get-Content -LiteralPath (Join-Path $repoRoot 'spikes/codex-app-server/fixtures/irl-cases.json') -Raw | ConvertFrom-Json
$drive = Get-PSDrive -Name D
$head = (& git -C $repoRoot rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0) { throw 'Cannot identify repository HEAD.' }
$result = [ordered]@{
    status = 'preparation_only'
    startedUtc = $started
    endedUtc = [DateTimeOffset]::UtcNow.ToString('o')
    repository = $repoRoot
    head = $head
    powershell = $PSVersionTable.PSVersion.ToString()
    scriptSha256 = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant()
    runnerProcesses = $runner
    runnerAbsentAtObservation = ($runner.Count -eq 0)
    userQuietWindowConfirmationReceived = $false
    gpuMeasurementStarted = $false
    commands = $commands
    expectedEngineFiles = $artifacts
    expectedModelFiles = $models
    driveDFreeBytes = $drive.Free
    fixtureCount = @($fixtures.cases).Count
    fixtureLanguages = @($fixtures.cases | Group-Object lang | ForEach-Object {
        [pscustomobject]@{ language = $_.Name; count = $_.Count }
    })
    inputBindings = $inputs
    limitations = @(
        'Expected paths and PATH are checked, not a whole-machine model search.',
        'No engine, injection, schema decoding, latency or VRAM measurement was run.',
        'Runner absence alone is not user authorization for a GPU window.'
    )
}
New-Item -ItemType Directory -Path $outputRoot | Out-Null
$result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $outputRoot 'inventory.json') -Encoding utf8
$result | ConvertTo-Json -Depth 8
