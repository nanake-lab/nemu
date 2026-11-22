# Resolve the Ubuntu check selection from the GitHub event and checked-out history.
# A manual run or an unavailable comparison runs everything (fail closed).
$full = $true
$files = @()
if ($env:EVENT -eq 'pull_request') {
  $files = @(git diff --name-only --no-renames "$($env:PR_BASE)...$($env:PR_HEAD)")
  $full = $LASTEXITCODE -ne 0
} elseif ($env:EVENT -eq 'push' -and $env:BEFORE -and $env:BEFORE -notmatch '^0+$') {
  $files = @(git diff --name-only --no-renames $env:BEFORE HEAD)
  $full = $LASTEXITCODE -ne 0
}
$scope = @{ node = $false; python = $false; rust = $false; storage = $false; contracts = $false; dotnet = $false; status = $false; causedemo = $false }
foreach ($f in $files) {
  if ($f -match '\.md$|^(docs/|tasks/|reports/|content$)') { continue }
  if ($f -match '^\.github/workflows/') { $full = $true; continue }
  if ($f -match '^tools/cause-demo/') { $scope.causedemo = $true; continue }
  $known = $false
  if ($f -match '^(apps/|packages/|tools/|spikes/|executors/codex-bridge/)') {
    $scope.node = $true; $known = $true
  }
  if ($f -match '^(workers/|services/)') { $scope.python = $true; $known = $true }
  if ($f -match '^core/') { $scope.rust = $true; $known = $true }
  # core-store currently depends on core-types and frozen contract fixtures.
  # Include workspace/lock/toolchain/config changes; update this with new path deps.
  if ($f -match '^core/(crates/(core-store|core-types)/|contracts/|\.cargo/|[^/]+$)') {
    $scope.storage = $true
  }
  if ($f -match '^(core/(contracts/|crates/core-types/|\.cargo/|[^/]+$)|packages/contracts/|workers/irl/generated/)') {
    $scope.contracts = $true
  }
  if ($f -match '^executors/office-connector/') { $scope.dotnet = $true; $known = $true }
  if ($f -match '^tools/status/') { $scope.status = $true }
  # New directories and shared root build/toolchain files cannot silently miss checks.
  if (-not $known) { $full = $true }
}
if ($full) { foreach ($key in @($scope.Keys)) { $scope[$key] = $true } }
# Contract/generated-type edits also exercise every Python consumer.
# Three-language conformance needs both the Node and Python toolchains.
if ($scope.contracts) { $scope.node = $true; $scope.python = $true }
foreach ($key in ($scope.Keys | Sort-Object)) {
  $value = $scope[$key].ToString().ToLowerInvariant()
  "$key=$value" >> $env:GITHUB_OUTPUT
  $message = if ($scope[$key]) { 'run' } else { '无相关改动' }
  "$key`: $message" | Tee-Object -FilePath $env:GITHUB_STEP_SUMMARY -Append
}
$tools = $scope.node -or $scope.python -or $scope.dotnet -or $scope.status -or $scope.causedemo
"tools=$($tools.ToString().ToLowerInvariant())" >> $env:GITHUB_OUTPUT
"full=$full; changed files=$($files.Count)"
# A failed diff selected the full suite; do not propagate its native exit code.
$global:LASTEXITCODE = 0
