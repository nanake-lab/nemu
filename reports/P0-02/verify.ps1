$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
Set-Location $root
$evidence = Join-Path $PSScriptRoot 'validation'
New-Item -ItemType Directory -Force $evidence | Out-Null
$checks = [Collections.Generic.List[object]]::new()
function Check([string]$Name, [scriptblock]$Run) {
    & $Run *>&1 | Tee-Object -FilePath (Join-Path $evidence "$Name.log")
    $code = $LASTEXITCODE
    $checks.Add([pscustomobject]@{ check = $Name; exitCode = $code })
    if ($code -ne 0) { throw "$Name failed: $code" }
}
try {
    Check 'install' { pnpm install --frozen-lockfile }
    Check 'lint' { pnpm lint }
    Check 'typecheck' { pnpm typecheck }
    Check 'unit' { pnpm test }
    Check 'build' { pnpm build }
    Check 'coverage' { pnpm test:contracts }
    Check 'format' { pnpm exec prettier --check packages/contracts package.json .github/workflows/ci.yml }
    Push-Location core
    try { Check 'rust-test' { cargo test --workspace --locked } } finally { Pop-Location }
    foreach ($project in @('workers/irl', 'workers/drive', 'workers/model-adapter', 'services/voice-sidecar')) {
        Push-Location $project
        try { Check ($project.Replace('/', '-') + '-test') { uv run pytest } } finally { Pop-Location }
    }
} finally {
    $checks | ConvertTo-Json | Out-File -Encoding utf8 (Join-Path $evidence 'checks.json')
}
