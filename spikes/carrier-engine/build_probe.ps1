param(
    [Parameter(Mandatory)][string]$OutputDirectory,
    [string]$SourceDirectory = 'D:/Dev/cache/P0-10/source/llama.cpp-b11669',
    [string]$SourceInventory = 'source-inventory-20261025T171354189657Z.json'
)
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$output = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $output) { throw 'Use a new build directory; preserve previous evidence' }
if (!$output.StartsWith('D:\Dev\cache\P0-10\', [StringComparison]::OrdinalIgnoreCase)) { throw 'Build outside task cache' }
$source = [IO.Path]::GetFullPath($SourceDirectory)
$inventory = Get-Content -LiteralPath (Join-Path $source $SourceInventory) -Raw | ConvertFrom-Json
if ($inventory.commit -ne '620d2225cc436ab0b001e3200e590e3b286ef9ab') { throw 'Wrong pinned headers' }
foreach ($entry in $inventory.files) {
    if ((Get-FileHash -LiteralPath (Join-Path $source $entry.path) -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.sha256) {
        throw "Changed upstream source: $($entry.path)"
    }
}
$compilerRoot = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools'
$vcvars = Join-Path $compilerRoot 'VC/Auxiliary/Build/vcvars64.bat'
$compiler = Join-Path $compilerRoot 'VC/Tools/MSVC/14.44.35207/bin/Hostx64/x64/cl.exe'
foreach ($path in @($root, $source, $output, $vcvars, $compiler)) {
    if ($path.Contains('"') -or $path.Contains("`n") -or $path.Contains('%')) { throw 'Unsafe command path' }
}
New-Item -ItemType Directory -Path $output | Out-Null
$unit = Join-Path $PSScriptRoot 'injection_probe.cpp'
$include = Join-Path $source 'include'
$ggmlInclude = Join-Path $source 'ggml/include'
$command = @"
@echo off
call "$vcvars" >nul
if errorlevel 1 exit /b %errorlevel%
"$compiler" /nologo /std:c++17 /EHsc /W4 /WX /O2 /fp:precise /utf-8 /I"$include" /I"$ggmlInclude" "$unit" /Fe:injection_probe.exe
exit /b %errorlevel%
"@
$commandFile = Join-Path $output 'compile.cmd'
[IO.File]::WriteAllText($commandFile, $command, [Text.Encoding]::ASCII)
$env:TMP = 'D:/Dev/cache/P0-10/tmp'
$env:TEMP = $env:TMP
$start = [DateTimeOffset]::UtcNow
Push-Location $output
try {
    & cmd.exe /d /c $commandFile *> (Join-Path $output 'compile.log')
    $code = $LASTEXITCODE
} finally { Pop-Location }
$binding = [ordered]@{
    exitCode = $code; startedUtc = $start.ToString('o'); endedUtc = [DateTimeOffset]::UtcNow.ToString('o')
    compiler = $compiler; compilerVersion = (Get-Item -LiteralPath $compiler).VersionInfo.FileVersion
    compilerSha256 = (Get-FileHash -LiteralPath $compiler).Hash.ToLowerInvariant()
    upstreamInventory = $SourceInventory
    sources = @('injection_probe.cpp', 'native_probe.hpp', 'activation_trace.hpp', 'build_probe.ps1') | ForEach-Object {
        [ordered]@{path = "spikes/carrier-engine/$_"; sha256 = (Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $_)).Hash.ToLowerInvariant()}
    }
}
if ($code -eq 0) { $binding.executableSha256 = (Get-FileHash -LiteralPath (Join-Path $output 'injection_probe.exe')).Hash.ToLowerInvariant() }
$binding | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $output 'build.json') -Encoding utf8NoBOM
Get-Content -LiteralPath (Join-Path $output 'compile.log')
exit $code
