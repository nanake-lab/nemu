param([string]$EvidenceDirectory = 'reports/P0-01')
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
Set-Location $root
$evidence = [IO.Path]::GetFullPath((Join-Path $root $EvidenceDirectory))
New-Item -ItemType Directory -Force $evidence | Out-Null
$results = [Collections.Generic.List[object]]::new()
function Check([string]$Name, [scriptblock]$Run) {
    Write-Host "Checking $Name"
    $ErrorActionPreference = 'Continue' # Windows PowerShell wraps native stderr as ErrorRecord.
    & $Run *>&1 | Tee-Object -FilePath (Join-Path $evidence ($Name + '.log'))
    $code = $LASTEXITCODE
    $ErrorActionPreference = 'Stop'
    $results.Add([pscustomobject]@{ check = $Name; exitCode = $code })
    if ($code -ne 0) { throw "$Name failed: $code" }
}
try {
    Check 'pnpm-install' { pnpm install --frozen-lockfile }
    Check 'lint' { pnpm lint }
    Check 'typecheck' { pnpm typecheck }
    Check 'unit' { pnpm test }
    Check 'build' { pnpm build }
    Check 'electron' { pnpm test:e2e }
    Check 'visual' { pnpm test:visual }
    Check 'transport' { pnpm test:transport }
    Check 'windows-isolation' { powershell -NoProfile -File tools/scaffold/test-windows-isolation.ps1 }
    Push-Location core
    try {
        Check 'rust-format' { cargo fmt --all -- --check }
        Check 'rust-clippy' { cargo clippy --workspace --all-targets -- -D warnings }
        Check 'rust-test' { cargo test --workspace --locked }
    } finally { Pop-Location }
    foreach ($project in @('workers/irl','workers/drive','workers/model-adapter','services/voice-sidecar')) {
        Push-Location $project
        try {
            $label = $project.Replace('/', '-')
            Check "$label-sync" { uv sync --locked }
            Check "$label-lint" { uv run ruff check . }
            Check "$label-types" { uv run mypy --strict src tests }
            Check "$label-test" { uv run pytest }
            Check "$label-build" { uv build }
        } finally { Pop-Location }
    }
    Check 'dotnet-build' { dotnet build executors/office-connector/OfficeConnector.csproj --configuration Release }
    Check 'ci-budget' { pnpm ci:budget }
} finally {
    $results | ConvertTo-Json | Out-File -Encoding utf8 (Join-Path $evidence 'validation.json')
}
