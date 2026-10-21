$ErrorActionPreference = 'Stop'
if (Get-Process Runner.Worker -ErrorAction SilentlyContinue) { throw 'Runner.Worker is active; local checks must wait.' }
$evidenceDirectory = $PSScriptRoot
$taskPython = Join-Path $PWD 'workers/irl/.venv/Scripts/python.exe'
$env:PYTHONPATH = Join-Path $PWD 'tools/asset-pipeline/.venv/Lib/site-packages'
$checks = @(
    @{ Name='pytest'; Exe=$taskPython; Args=@('-m','pytest','tools/asset-pipeline/tests','-q') },
    @{ Name='ruff'; Exe=$taskPython; Args=@('-m','ruff','check','tools/asset-pipeline','tools/texture-fix') },
    @{ Name='mypy'; Exe=$taskPython; Args=@('-m','mypy','--strict','--config-file','tools/asset-pipeline/pyproject.toml','tools/asset-pipeline','tools/texture-fix/texture_fix.py') },
    @{ Name='pnpm-lint'; Exe='pnpm'; Args=@('lint') },
    @{ Name='pnpm-typecheck'; Exe='pnpm'; Args=@('typecheck') },
    @{ Name='pnpm-test'; Exe='pnpm'; Args=@('test') }
)
$taskResults = @()
foreach ($check in $checks) {
    $taskStarted = [DateTimeOffset]::UtcNow.ToString('o')
    $taskArguments = $check.Args
    & $check.Exe @taskArguments 2>&1 | Tee-Object -FilePath (Join-Path $evidenceDirectory ($check.Name + '.log'))
    $taskExit = $LASTEXITCODE
    $taskResults += [pscustomobject]@{ name=$check.Name; executable=$check.Exe; arguments=$check.Args; startedUtc=$taskStarted; endedUtc=[DateTimeOffset]::UtcNow.ToString('o'); exitCode=$taskExit }
    $taskResults | ConvertTo-Json -Depth 5 | Set-Content -Encoding utf8 (Join-Path $evidenceDirectory 'local-checks.json')
    if ($taskExit -ne 0) { throw "Check $($check.Name) failed with exit code $taskExit; original log retained." }
}
