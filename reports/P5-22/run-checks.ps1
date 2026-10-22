param([string]$Evidence = 'reports/P5-22/checks')
$ErrorActionPreference = 'Stop'
$env:PYTHONUTF8 = '1'
$env:UV_LINK_MODE = 'copy'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Set-Location -LiteralPath $root
$output = [IO.Path]::GetFullPath((Join-Path $root $Evidence))
New-Item -ItemType Directory -Force -Path $output | Out-Null
$results = [Collections.Generic.List[object]]::new()
function Check([string]$Name, [string]$Directory, [string]$Exe, [string[]]$Arguments) {
  $log = Join-Path $output ($Name + '.log')
  if (Test-Path -LiteralPath $log) { throw "Evidence already exists: $log" }
  $start = [DateTimeOffset]::UtcNow.ToString('o')
  Push-Location -LiteralPath (Join-Path $root $Directory)
  try {
    & $Exe @Arguments *> $log
    $code = $LASTEXITCODE
  } finally { Pop-Location }
  $results.Add([ordered]@{name=$Name;cwd=$Directory;command=(@($Exe)+$Arguments)-join ' ';started=$start;ended=[DateTimeOffset]::UtcNow.ToString('o');exit_code=$code;log=$log})
  $results | ConvertTo-Json -Depth 5 | Set-Content -Encoding utf8 (Join-Path $output 'results.json')
  Write-Output "$Name exit=$code"
}
Check 'generate' '.' 'python' @('core/contracts/generate.py','--check')
Check 'compat' '.' 'python' @('core/contracts/check_compat.py','--ref','origin/main')
Check 'parser' '.' 'python' @('-m','unittest','discover','-s','core/contracts','-p','test_*schema.py')
Check 'lint' '.' 'pnpm.cmd' @('lint')
Check 'typecheck' '.' 'pnpm.cmd' @('typecheck')
Check 'unit' '.' 'pnpm.cmd' @('test')
Check 'contracts' '.' 'pnpm.cmd' @('test:contracts',('--coverage.reportsDirectory=' + $Evidence + '/coverage'))
Check 'build' '.' 'pnpm.cmd' @('build')
Check 'rust-fmt' 'core' 'cargo' @('fmt','--all','--','--check')
Check 'rust-clippy' 'core' 'cargo' @('clippy','--workspace','--all-targets','--','-D','warnings')
Check 'rust-test' 'core' 'cargo' @('test','--workspace','--locked')
Check 'core-contracts' '.' 'pnpm.cmd' @('test:core-contracts')
Check 'contract-ruff' '.' 'uv' @('run','--project','core/contracts','--locked','ruff','check','core/contracts','workers/irl/generated/echo')
Check 'contract-mypy' '.' 'uv' @('run','--project','core/contracts','--locked','mypy','--strict','core/contracts/generate.py','core/contracts/generate_browser.py','core/contracts/fixture_inputs.py','core/contracts/adversarial.py','core/contracts/verify.py','core/contracts/check_compat.py','core/contracts/freeze_vectors.py','core/contracts/test_oneof_schema.py','core/contracts/test_browser_schema.py','workers/irl/generated/echo')
foreach ($worker in @('irl','drive','model-adapter')) {
  Check "$worker-sync" "workers/$worker" 'uv' @('sync','--locked')
  Check "$worker-ruff" "workers/$worker" 'uv' @('run','--locked','ruff','check','.')
  Check "$worker-mypy" "workers/$worker" 'uv' @('run','--locked','mypy','--strict','src','tests')
  Check "$worker-pytest" "workers/$worker" 'uv' @('run','--locked','pytest')
}
if (@($results | Where-Object { $_.exit_code -ne 0 }).Count) { exit 1 }
