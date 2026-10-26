$ErrorActionPreference = 'Stop'
$repository = 'D:\Dev\nemu-wt\P5-04'
$checks = @('panel-budget-green','fmt','clippy','workspace-tests','fmt-main','clippy-main','workspace-tests-main')
foreach ($check in $checks) {
    $result = Get-Content -LiteralPath (Join-Path $PSScriptRoot ($check + '.json')) -Raw | ConvertFrom-Json
    if ($result.exit_code -ne 0) { throw ($check + ' failed') }
}
$negative = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'budget-regression-negative-control.json') -Raw | ConvertFrom-Json
if ($negative.actual_test_exit_code -ne 101 -or -not $negative.fixed_source_restored) { throw 'Negative control invalid' }
$sourceHash = (Get-FileHash -LiteralPath (Join-Path $repository 'core/crates/fact-view/src/paging.rs') -Algorithm SHA256).Hash.ToLowerInvariant()
if ($sourceHash -ne $negative.restored_source_sha256) { throw 'Production source changed after regression checks' }
$scope = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'ci-scope-result.json') -Raw | ConvertFrom-Json
if ($scope.cases.Count -ne 11 -or @($scope.cases | Where-Object result -ne 'passed').Count -ne 0) {
    throw 'CI scope regression mismatch'
}
$workspace = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'workspace-tests-main.log') -Raw
if ($workspace -notmatch '18 passed; 0 failed; 2 ignored' -or $workspace -notmatch '32 passed; 0 failed') {
    throw 'Expected fact-view and core-store regression results missing'
}
git -C $repository diff --quiet 092cb2fe -- core/contracts packages/contracts/src/generated workers/irl/generated/echo core/crates/core-store core/crates/core-types core/Cargo.lock reports/P5-04/windows-failure-fbcb7cc-20261026
if ($LASTEXITCODE -ne 0) { throw 'Protected files or original failure evidence changed' }
$windows = git -C $repository rev-parse 'HEAD:.github/workflows/windows.yml'
$mainWindows = git -C $repository rev-parse 'origin/main:.github/workflows/windows.yml'
if ($windows -ne $mainWindows -or $windows -ne 'a686643b9c6f7508e949157c36ddaf97e946e8fc') {
    throw 'Windows workflow differs from main'
}
git -C $repository diff --quiet origin/main -- .github/workflows/windows.yml
if ($LASTEXITCODE -ne 0) { throw 'Local Windows workflow edit' }
$paths = @('.github/workflows/ci.yml','core/crates/fact-view/src/paging.rs','core/crates/fact-view/src/tests/mod.rs','core/crates/fact-view/src/tests/panel_budget.rs')
$sourceHashes = @($paths | ForEach-Object {
    $path = Join-Path $repository $_
    $lines = (Get-Content -LiteralPath $path).Count
    if ($_ -like '*.rs' -and $lines -gt 400) { throw ('Rust file exceeds 400 lines: ' + $_) }
    [ordered]@{path=$_; lines=$lines; sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()}
})
[ordered]@{
    checked_at_utc=[DateTime]::UtcNow.ToString('o'); result='passed'; baseline='092cb2fe1979beec6b537290173684f5ea6ed86c'
    main_sync_commit='c4162dec56b8d515b507f325fdea53dbf507f140'; merged_main='65052c0c45f932b601340fca43318ba7db814f71'
    local_checks=$checks; negative_control_expected_failure=101; empty_response_combinations=45
    new_rust_tests=3; fact_view_passed=18; fact_view_existing_ignored=2; core_store_passed=32
    ci_scope_scenarios=11; source_files=$sourceHashes; windows_workflow_blob=$windows
    frozen_and_protected_files_changed=0; original_failure_evidence_changed=0; ci_retried=$false
} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'verification.json') -Encoding utf8
$hashes = Get-ChildItem -LiteralPath $PSScriptRoot -File -Recurse | Where-Object Name -ne 'SHA256.json' |
    Sort-Object FullName | ForEach-Object {
        [ordered]@{path=[IO.Path]::GetRelativePath($PSScriptRoot,$_.FullName).Replace('\','/'); bytes=$_.Length
            sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}
    }
$hashes | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'SHA256.json') -Encoding utf8
'All local check results, regression provenance, source scope and evidence hashes verified.'
