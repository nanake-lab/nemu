# Resolve Windows checks for the complete PR, push, or branch.
# GitHub's paths filter and these group patterns must stay aligned.
$full = $true
$files = @()
switch ($env:EVENT) {
    'pull_request' {
        if ($env:PR_BASE -and $env:PR_HEAD) {
            $files = @(git -c core.quotepath=false diff --name-only --no-renames "$($env:PR_BASE)...$($env:PR_HEAD)")
            $full = $LASTEXITCODE -ne 0
        }
    }
    'push' {
        if ($env:BEFORE -and $env:BEFORE -notmatch '^0+$') {
            $files = @(git -c core.quotepath=false diff --name-only --no-renames $env:BEFORE HEAD)
            $full = $LASTEXITCODE -ne 0
        }
    }
    'workflow_dispatch' {
        if ($env:FULL_INPUT -ne 'true' -and $env:DEFAULT_REF) {
            $files = @(git -c core.quotepath=false diff --name-only --no-renames "$($env:DEFAULT_REF)...HEAD")
            $full = $LASTEXITCODE -ne 0
        }
    }
}
# An escaped path may hide a relevant name. Fall back to all checks.
$full = $full -or (($files -match '^"').Count -gt 0)
$metadata = '(?:\.md$|(?:^|/)(?:manifest\.json|(?:licen[cs]e(?:-(?:mit|apache|bsd))?|copying(?:\.lesser)?|notice)(?:\.(?:md|txt|rst))?)$)'
$metadataOnly = $files.Count -gt 0 -and ($files -notmatch $metadata).Count -eq 0

# For mixed changes, keep the existing path routing.
$files = @($files | Where-Object { $_ -and $_ -notmatch '\.md$' })
$workflow = '^\.github/workflows/windows\.yml$|^tools/scaffold/windows-ci-scope\.ps1$'
$desktopPaths = @(
    '^apps/', '^packages/', '^executors/codex-bridge/',
    '^workers/[^/]+/src/', '^services/[^/]+/src/',
    '^tools/(?!status/|cause-demo/|asset-pipeline/|texture-fix/|scaffold/ci-budget\.mjs$)',
    '^(package\.json|pnpm-lock\.yaml|pnpm-workspace\.yaml|\.nvmrc|\.npmrc|\.gitattributes)$',
    '^tsconfig[^/]*\.json$', '^executors/office-connector/global\.json$', $workflow
) -join '|'
$storagePaths = @(
    '^core/crates/core-store/', '^core/crates/core-types/', '^core/contracts/',
    '^core/(Cargo\.toml|Cargo\.lock|rust-toolchain\.toml)$', $workflow
) -join '|'
$gatewayPaths = '^core/crates/gateway/|' + $storagePaths
$scope = @{
    desktop = $full -or (-not $metadataOnly -and ($files -match $desktopPaths).Count -gt 0)
    storage = $full -or ($files -match $storagePaths).Count -gt 0
    gateway = $full -or ($files -match $gatewayPaths).Count -gt 0
}
$scope.self_hosted = $scope.desktop -or $scope.gateway
foreach ($key in ($scope.Keys | Sort-Object)) {
    "$key=$($scope[$key].ToString().ToLowerInvariant())" >> $env:GITHUB_OUTPUT
}
"full=$full; metadataOnly=$metadataOnly; desktop=$($scope.desktop); storage=$($scope.storage); gateway=$($scope.gateway)"
# A failed comparison deliberately selects all checks.
$global:LASTEXITCODE = 0
