param([Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference = 'Stop'
$output = [IO.Path]::GetFullPath($OutputDirectory)
$cache = [IO.Path]::GetFullPath('D:/Dev/cache/R-CHOICE-01') + [IO.Path]::DirectorySeparatorChar
if (!$output.StartsWith($cache, [StringComparison]::OrdinalIgnoreCase)) { throw 'Use the experiment D-drive cache' }
if (Test-Path -LiteralPath $output) { throw 'Preserve existing build evidence' }
$source = 'D:/Dev/cache/P0-10/source/llama.cpp-b11669'
$inventory = Get-Content -Raw (Join-Path $source 'source-inventory-20261025T171354189657Z.json') | ConvertFrom-Json
if ($inventory.commit -ne '620d2225cc436ab0b001e3200e590e3b286ef9ab') { throw 'Wrong pinned headers' }
foreach ($entry in $inventory.files) {
    if ((Get-FileHash -LiteralPath (Join-Path $source $entry.path)).Hash.ToLowerInvariant() -ne $entry.sha256) { throw "Changed header: $($entry.path)" }
}
$vs = 'C:/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools'
$vcvars = Join-Path $vs 'VC/Auxiliary/Build/vcvars64.bat'
$compiler = Join-Path $vs 'VC/Tools/MSVC/14.44.35207/bin/Hostx64/x64/cl.exe'
$unit = Join-Path $PSScriptRoot 'choice_probe.cpp'
New-Item -ItemType Directory -Path $output | Out-Null
$temp = Join-Path $output 'tmp'
New-Item -ItemType Directory -Path $temp | Out-Null
$env:TEMP = $temp
$env:TMP = $temp
$command = @"
@echo off
call "$vcvars" >nul
if errorlevel 1 exit /b %errorlevel%
"$compiler" /nologo /std:c++17 /EHsc /W4 /WX /O2 /fp:precise /utf-8 /I"$source/include" /I"$source/ggml/include" "$unit" /Fe:choice_probe.exe
exit /b %errorlevel%
"@
$commandFile = Join-Path $output 'compile.cmd'
[IO.File]::WriteAllText($commandFile, $command, [Text.Encoding]::ASCII)
Push-Location $output
try { & cmd.exe /d /c $commandFile *> (Join-Path $output 'compile.log'); $code = $LASTEXITCODE } finally { Pop-Location }
$binding = [ordered]@{exitCode=$code; compiler=$compiler; compilerSha256=(Get-FileHash $compiler).Hash.ToLowerInvariant(); sourceSha256=(Get-FileHash $unit).Hash.ToLowerInvariant(); baseline='f24f1256259c998bc213a9dc93103fcfb9cfe89a'}
if ($code -eq 0) { $binding.executableSha256 = (Get-FileHash (Join-Path $output 'choice_probe.exe')).Hash.ToLowerInvariant() }
$binding | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $output 'build.json') -Encoding utf8NoBOM
Get-Content -LiteralPath (Join-Path $output 'compile.log')
exit $code
