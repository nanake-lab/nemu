$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$evidence = Join-Path $PSScriptRoot 'acceptance'
New-Item -ItemType Directory -Force -Path $evidence | Out-Null
$results = [System.Collections.Generic.List[object]]::new()
function Check([string]$Name, [string]$Directory, [string]$Exe, [string[]]$Arguments) {
    Write-Host "Checking $Name"
    Push-Location $Directory
    try {
        & $Exe @Arguments *> (Join-Path $evidence "$Name.log")
        $code = $LASTEXITCODE
        $results.Add([pscustomobject]@{name=$Name; command=($Exe+' '+($Arguments -join ' ')); exit_code=$code})
        $results | ConvertTo-Json -Depth 4 | Set-Content -Encoding utf8 (Join-Path $evidence 'checks.json')
        if ($code -ne 0) { throw "$Name failed: see acceptance/$Name.log" }
    } finally { Pop-Location }
}
Check 'D10.8-SYS21' $repo 'python' @('core/contracts/verify.py')
Check 'rust-fmt' (Join-Path $repo 'core') 'cargo' @('fmt','--all','--','--check')
Check 'rust-clippy' (Join-Path $repo 'core') 'cargo' @('clippy','--workspace','--all-targets','--locked','--','-D','warnings')
Check 'rust-test' (Join-Path $repo 'core') 'cargo' @('test','--workspace','--locked')
foreach ($command in @('lint','typecheck','test','build','test:contracts')) {
    Check ('pnpm-'+$command.Replace(':','-')) $repo 'pnpm' @($command)
}
Check 'contract-ruff' $repo 'uv' @('run','--project','core/contracts','--locked','ruff','check','core/contracts','workers/irl/generated/echo')
Check 'contract-mypy' $repo 'uv' @('run','--project','core/contracts','--locked','mypy','--strict','core/contracts/generate.py','core/contracts/fixture_inputs.py','core/contracts/adversarial.py','core/contracts/verify.py','workers/irl/generated/echo')
foreach ($project in @('workers/irl','workers/drive','workers/model-adapter','services/voice-sidecar')) {
    $name = $project.Replace('/','-')
    $directory = Join-Path $repo $project
    Check "$name-ruff" $directory 'uv' @('run','--locked','ruff','check','.')
    Check "$name-mypy" $directory 'uv' @('run','--locked','mypy','--strict','src','tests')
    Check "$name-test" $directory 'uv' @('run','--locked','pytest')
    if ($project.StartsWith('workers/')) {
        Check "$name-generated-mypy" $directory 'uv' @('run','--locked','mypy','--strict','generated/echo')
    }
}
Check 'frozen-specs' $repo 'git' @('diff','--exit-code','origin/main','--','docs/echo-core/original','docs/ARCHITECTURE.md','docs/CONTRACTS.md','docs/adr/0006-single-root-spec-conflict.md')
Write-Host "All $($results.Count) local checks passed. CI remains a separate merge gate."
