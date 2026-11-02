param(
    [Parameter(Mandatory = $true)][string]$WorkRoot
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not $IsWindows) { throw 'Windows required.' }
if (-not [IO.Path]::IsPathFullyQualified($WorkRoot)) { throw 'WorkRoot must be absolute.' }
$testRoot = [IO.Path]::GetFullPath($WorkRoot)
if (Test-Path -LiteralPath $testRoot) { throw 'WorkRoot must be new; previous evidence is preserved.' }
# Refuse to contend with another build or CI job; caller can retry after it ends.
$busy = Get-Process cargo,rustc,dotnet,Runner.Worker -ErrorAction SilentlyContinue
if ($busy) { throw 'Build or CI work is active. No tests started.' }
$null = New-Item -ItemType Directory -Path $testRoot
$env:DOTNET_CLI_HOME = Join-Path $testRoot 'dotnet-home'
$env:NUGET_PACKAGES = Join-Path $testRoot 'nuget'
$env:DOTNET_GENERATE_ASPNET_CERTIFICATE = 'false'
$env:DOTNET_ADD_GLOBAL_TOOLS_TO_PATH = 'false'
$env:DOTNET_CLI_TELEMETRY_OPTOUT = '1'
$env:DOTNET_SKIP_FIRST_TIME_EXPERIENCE = '1'
$env:TEMP = Join-Path $testRoot 'temp'
$env:TMP = $env:TEMP
$null = New-Item -ItemType Directory -Path $env:TEMP
$launcherProject = Join-Path $PSScriptRoot 'LowIlLauncher.csproj'
$testProject = Join-Path $PSScriptRoot 'tests/IsolationTests.csproj'
$buildRoot = Join-Path $testRoot 'build'
Push-Location $PSScriptRoot
try {
    & dotnet build $launcherProject --configuration Release --artifacts-path $buildRoot --disable-build-servers 2>&1 |
        Tee-Object -FilePath (Join-Path $testRoot 'build-launcher.log')
    if ($LASTEXITCODE -ne 0) { throw "Launcher build failed: $LASTEXITCODE" }
    & dotnet build $testProject --configuration Release --artifacts-path $buildRoot --disable-build-servers 2>&1 |
        Tee-Object -FilePath (Join-Path $testRoot 'build-tests.log')
    if ($LASTEXITCODE -ne 0) { throw "Test build failed: $LASTEXITCODE" }
    $launcher = Join-Path $buildRoot 'bin/LowIlLauncher/release/LowIlLauncher.exe'
    $tests = Join-Path $buildRoot 'bin/IsolationTests/release/IsolationTests.exe'
    & $tests $launcher (Join-Path $testRoot 'isolation') 2>&1 |
        Tee-Object -FilePath (Join-Path $testRoot 'isolation.log')
    if ($LASTEXITCODE -ne 0) { throw "Isolation tests failed: $LASTEXITCODE" }
}
finally { Pop-Location }
