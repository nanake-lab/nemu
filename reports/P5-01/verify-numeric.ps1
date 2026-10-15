$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$evidence = Join-Path $PSScriptRoot 'verification'
New-Item -ItemType Directory -Force -Path $evidence | Out-Null
$checks = [System.Collections.Generic.List[object]]::new()
Push-Location (Join-Path $repoRoot 'core')
try {
    & cargo fmt --all -- --check *> (Join-Path $evidence 'fmt.log')
    $checks.Add(@{ check = 'cargo fmt --all -- --check'; exitCode = $LASTEXITCODE })
    & cargo clippy --workspace --all-targets --locked -- -D warnings *> (Join-Path $evidence 'clippy.log')
    $checks.Add(@{ check = 'cargo clippy --workspace --all-targets --locked -- -D warnings'; exitCode = $LASTEXITCODE })
    & cargo test --workspace --locked *> (Join-Path $evidence 'test.log')
    $checks.Add(@{ check = 'cargo test --workspace --locked'; exitCode = $LASTEXITCODE })
} finally { Pop-Location }
Push-Location $repoRoot
try {
    & git diff --check *> (Join-Path $evidence 'diff-check.log')
    $checks.Add(@{ check = 'git diff --check'; exitCode = $LASTEXITCODE })
    & git diff --exit-code origin/main -- docs/echo-core docs/ARCHITECTURE.md docs/CONTRACTS.md docs/adr/0006-single-root-spec-conflict.md *> (Join-Path $evidence 'frozen-diff.log')
    $checks.Add(@{ check = 'frozen specifications and ADR 0006 unchanged'; exitCode = $LASTEXITCODE })
} finally { Pop-Location }
$checks | ConvertTo-Json | Set-Content -Encoding utf8 (Join-Path $evidence 'checks.json')
$checks | Format-Table
if ($checks.Where({ $_.exitCode -ne 0 }).Count -gt 0) { exit 1 }
