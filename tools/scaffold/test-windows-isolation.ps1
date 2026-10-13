$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT') { throw 'Windows runner required' }
$taskTemp = Join-Path ([IO.Path]::GetTempPath()) ('nemu-isolation-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $taskTemp | Out-Null
try {
    & icacls.exe $taskTemp /setintegritylevel '(OI)(CI)M' | Out-Host
    if ($LASTEXITCODE -ne 0) { throw 'Could not set medium integrity on test directory' }
    & dotnet run --project (Join-Path $PSScriptRoot 'windows-isolation/IsolationSmoke.csproj') --configuration Release -- $taskTemp
    if ($LASTEXITCODE -ne 0) { throw 'Integrity smoke failed' }
} finally {
    $resolved = [IO.Path]::GetFullPath($taskTemp)
    $tempRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    if (!$resolved.StartsWith($tempRoot, [StringComparison]::OrdinalIgnoreCase) -or !(Split-Path $resolved -Leaf).StartsWith('nemu-isolation-')) { throw 'Unsafe cleanup path' }
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
