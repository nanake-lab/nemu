param([Parameter(Mandatory = $true)][string]$Output)
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$manifest = Join-Path $repo 'core/Cargo.toml'
$out = [IO.Path]::GetFullPath($Output)
if (Test-Path -LiteralPath $out) { throw "Evidence directory already exists: $out" }
New-Item -ItemType Directory -Path $out | Out-Null
$records = [Collections.Generic.List[object]]::new()
$kernelTest = 'dynamics::storage::tests::actual_states_all_prefixes_roundtrip_and_continuation'
function Invoke-Check([string]$Name, [string[]]$Arguments) {
    $start = [DateTime]::UtcNow.ToString('o')
    & cargo @Arguments *> (Join-Path $out "$Name.log")
    $code = $LASTEXITCODE
    $records.Add(@{ name = $Name; command = @('cargo') + $Arguments; cwd = $repo; start = $start; end = [DateTime]::UtcNow.ToString('o'); exit = $code })
    $records | ConvertTo-Json -Depth 8 | Set-Content -Encoding utf8 (Join-Path $out 'commands.json')
    if ($code -ne 0) { throw "$Name failed: exit $code; raw log retained" }
}
Push-Location $repo
try {
    foreach ($key in @('P536_RESTORE','P536_EXPORT','P536_STORE_INPUT','P536_STORE_OUTPUT','P536_STORE_RESTORE')) {
        if (Test-Path "Env:$key") { throw "Unexpected inherited acceptance setting: $key" }
    }
    $env:P536_EXPORT = Join-Path $out 'actual-states.json'
    Invoke-Check 'export' @('test','--manifest-path',$manifest,'-p','core-kernel','--locked',$kernelTest,'--','--exact','--nocapture')
    Remove-Item Env:P536_EXPORT
    $env:P536_STORE_INPUT = Join-Path $out 'actual-states.json'
    $env:P536_STORE_OUTPUT = Join-Path $out 'stored-states.json'
    Invoke-Check 'store' @('test','--manifest-path',$manifest,'-p','core-store','--locked','tests::theta::actual_states_commit_close_reopen','--','--exact','--nocapture','--test-threads=1')
    Remove-Item Env:P536_STORE_INPUT, Env:P536_STORE_OUTPUT
    $env:P536_STORE_RESTORE = Join-Path $out 'stored-states.json'
    Invoke-Check 'restore-continue' @('test','--manifest-path',$manifest,'-p','core-kernel','--locked',$kernelTest,'--','--exact','--nocapture')
    @{ result = 'passed'; prefixes = 65; oracle = 'exhaustive private fields, f64::to_bits'; boundary = 'commit -> close -> reopen -> read -> restore -> every remaining event'; source = 'existing P5-36 C trajectory'; noMigration = $true } | ConvertTo-Json | Set-Content -Encoding utf8 (Join-Path $out 'result.json')
} finally {
    foreach ($key in @('P536_RESTORE','P536_EXPORT','P536_STORE_INPUT','P536_STORE_OUTPUT','P536_STORE_RESTORE')) {
        Remove-Item "Env:$key" -ErrorAction SilentlyContinue
    }
    Pop-Location
}
