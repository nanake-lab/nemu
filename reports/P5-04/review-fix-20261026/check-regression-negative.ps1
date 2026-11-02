$ErrorActionPreference = 'Stop'
$repository = 'D:\Dev\nemu-wt\P5-04'
$sourcePath = Join-Path $repository 'core/crates/fact-view/src/paging.rs'
$fixedBytes = [IO.File]::ReadAllBytes($sourcePath)
$fixedHash = (Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash
$oldLines = git -C $repository show '092cb2fe1979beec6b537290173684f5ea6ed86c:core/crates/fact-view/src/paging.rs'
if ($LASTEXITCODE -ne 0) { throw 'Cannot read reviewed implementation' }
$baselineBytes = [Text.UTF8Encoding]::new($false).GetBytes(($oldLines -join "`n") + "`n")
$busy = @(Get-Process -Name cargo,rustc,core_store*,fact_view*,'Runner.Worker' -ErrorAction SilentlyContinue)
if ($busy.Count -gt 0) { throw 'Heavy resource window occupied; no source changed or test started' }
try {
    # Fault injection: pair the new regression with exactly the old implementation.
    [IO.File]::WriteAllBytes($sourcePath, $baselineBytes)
    & (Join-Path $PSScriptRoot 'run-check.ps1') -Name 'budget-regression-red' -Tool cargo -Arguments @(
        'test','--manifest-path','core/Cargo.toml','-p','fact-view','--release',
        '--features','cpu-embedding','sys05_empty_panel_statuses_enforce_the_entire_response_envelope',
        '--locked','--','--nocapture','--test-threads=1'
    )
    $code = $LASTEXITCODE
    $log = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'budget-regression-red.log') -Raw
    if ($code -ne 101 -or $log -notmatch 'must reject an envelope requiring') {
        throw 'Negative control did not fail at the targeted budget assertion'
    }
} finally {
    [IO.File]::WriteAllBytes($sourcePath, $fixedBytes)
    if ((Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash -ne $fixedHash) {
        throw 'Fixed source restoration failed'
    }
}
[ordered]@{
    checked_at_utc=[DateTime]::UtcNow.ToString('o'); result='passed'
    baseline='092cb2fe1979beec6b537290173684f5ea6ed86c'; expected_test_exit_code=101
    actual_test_exit_code=$code; failure_assertion='must reject an envelope requiring'
    fixed_source_restored=$true; restored_source_sha256=$fixedHash.ToLowerInvariant()
} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'budget-regression-negative-control.json') -Encoding utf8
'Negative control rejected the old implementation; fixed source restored exactly.'
