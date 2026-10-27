param(
    [Parameter(Mandatory)][string]$Name,
    [Parameter(Mandatory)][string]$Tool,
    [string[]]$Arguments = @(),
    [string]$WorkingDirectory = 'D:\Dev\nemu-wt\P5-04'
)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$busy = @(Get-Process -Name cargo,rustc,core_store*,fact_view*,'Runner.Worker' -ErrorAction SilentlyContinue)
if ($busy.Count -gt 0) { throw ('Heavy resource window occupied: ' + (($busy | ForEach-Object { "$($_.ProcessName):$($_.Id)" }) -join ', ')) }
$log = Join-Path $PSScriptRoot ($Name + '.log')
$result = Join-Path $PSScriptRoot ($Name + '.json')
if ((Test-Path -LiteralPath $log) -or (Test-Path -LiteralPath $result)) { throw 'Preserve existing evidence; use a new check name' }
Push-Location -LiteralPath $WorkingDirectory
try {
    . 'D:\Dev\nemu-wt\P5-04\core\crates\fact-view\scripts\env.ps1'
    $start = Get-Date
    & $Tool @Arguments *> $log
    $code = $LASTEXITCODE
    [ordered]@{name=$Name;tool=$Tool;arguments=$Arguments;cwd=$WorkingDirectory;started_at=$start.ToString('o');finished_at=(Get-Date).ToString('o');exit_code=$code;temporary_directory=[IO.Path]::GetTempPath()} | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $result -Encoding utf8
} finally { Pop-Location }
Get-Content -LiteralPath $log -Tail 32
exit $code
