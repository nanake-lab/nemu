param([Parameter(Mandatory=$true)][string]$Attempt)
$ErrorActionPreference = 'Stop'
$env:PYTHONUTF8 = '1'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$dest = Join-Path $PSScriptRoot "verification/$Attempt"
if (Test-Path -LiteralPath $dest) { throw "Refuse to overwrite evidence: $dest" }
New-Item -ItemType Directory -Path $dest | Out-Null
Push-Location $root
try {
    $commands = @(
        @{name='old'; argv=@('-B','reports/echo-core-v2-draft/check_law.py')},
        @{name='new-legacy'; argv=@('-B','reports/echo-core-v2-draft/check_law_v2.py','--suite','legacy')},
        @{name='new-extra'; argv=@('-B','reports/echo-core-v2-draft/check_law_v2.py','--suite','extra')},
        @{name='worlds'; argv=@('-B','reports/echo-core-v2-draft/compare_law_v2.py','--out',"$dest/worlds")}
    )
    foreach ($item in $commands) {
        $started = [DateTimeOffset]::UtcNow.ToString('o')
        $argv = $item.argv
        & python @argv 1> (Join-Path $dest "$($item.name).stdout.txt") 2> (Join-Path $dest "$($item.name).stderr.txt")
        $code = $LASTEXITCODE
        [ordered]@{command="python $($argv -join ' ')"; cwd=$root; started_utc=$started;
            ended_utc=[DateTimeOffset]::UtcNow.ToString('o'); exit_code=$code} |
            ConvertTo-Json | Set-Content -Encoding utf8 (Join-Path $dest "$($item.name).meta.json")
        Write-Output "$($item.name): exit $code"
    }
} finally { Pop-Location }
