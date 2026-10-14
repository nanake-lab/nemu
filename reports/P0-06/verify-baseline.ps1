#requires -Version 7.0
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
Set-Location $root
$output = Join-Path $PSScriptRoot ('verification-' + [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffZ'))
New-Item -ItemType Directory $output | Out-Null
$checks = [Collections.Generic.List[object]]::new()
function Check([string]$Name, [scriptblock]$Run) {
    & $Run *>&1 | Tee-Object (Join-Path $output "$Name.log")
    $code = $LASTEXITCODE
    $checks.Add([pscustomobject]@{ check = $Name; exitCode = $code })
    if ($code -ne 0) { throw "$Name failed: $code" }
}
try {
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $baseline = Get-Content docs/traceability/baseline.md -Raw -Encoding utf8
    $originals = @(Get-ChildItem docs/echo-core/original -Filter *.docx | Sort-Object Name)
    if ($originals.Count -ne 5) { throw 'Expected exactly five baseline originals' }
    $sources = foreach ($file in $originals) {
        $hash = (Get-FileHash $file.FullName -Algorithm SHA256).Hash
        if (-not $baseline.Contains($hash) -or -not $baseline.Contains($file.Name)) { throw 'Baseline source hash/name mismatch' }
        $zip = [IO.Compression.ZipFile]::OpenRead($file.FullName)
        try {
            $reader = [IO.StreamReader]::new($zip.GetEntry('word/document.xml').Open())
            try { [xml]$document = $reader.ReadToEnd() } finally { $reader.Dispose() }
            $ns = [Xml.XmlNamespaceManager]::new($document.NameTable)
            $ns.AddNamespace('w', 'http://schemas.openxmlformats.org/wordprocessingml/2006/main')
            $paragraphs = @($document.SelectNodes('//w:body/w:p', $ns) | ForEach-Object {
                ($_.SelectNodes('.//w:t', $ns) | ForEach-Object InnerText) -join ''
            } | Where-Object { $_ })
            [pscustomobject]@{
                file = $file.Name; sha256 = $hash
                titleParagraphs = @($paragraphs | Select-Object -First 8)
                conflictParagraphs = @($paragraphs | Where-Object { $_ -match '单根|单一 root|24 项|24个|24 个|Eligible_single' })
            }
        } finally { $zip.Dispose() }
    }
    $sources | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $output 'originals.json') -Encoding utf8
    $scope = Get-Content docs/traceability/scope-v1.md -Raw -Encoding utf8
    foreach ($group in @(@('W',0,23), @('F',1,7), @('G',1,11), @('C',1,12), @('J',1,7), @('X',1,8), @('SYS',1,25), @('I',1,22))) {
        foreach ($number in $group[1]..$group[2]) {
            $id = '{0}{1:D2}' -f $group[0], $number
            if ($scope -notmatch ('\b' + $id + '\b')) { throw "Missing routing ID $id" }
        }
    }
    $register = Get-Content docs/traceability/diff-register.md -Raw -Encoding utf8
    foreach ($number in 1..14) {
        if ($register -notmatch ("(?m)^\| E$number \|")) { throw "Missing E$number" }
    }
    $locks = @('pnpm-lock.yaml','core/Cargo.lock','core/rust-toolchain.toml','executors/office-connector/global.json',
        'workers/irl/uv.lock','workers/drive/uv.lock','workers/model-adapter/uv.lock','services/voice-sidecar/uv.lock',
        'reports/P0-06/sqlite-probe/Cargo.lock')
    @($locks | ForEach-Object { [pscustomobject]@{ path = $_; sha256 = (Get-FileHash $_ -Algorithm SHA256).Hash } }) |
        ConvertTo-Json | Set-Content (Join-Path $output 'lock-hashes.json') -Encoding utf8
    $os = Get-CimInstance Win32_OperatingSystem
    [pscustomobject]@{
        node = (& node --version); pnpm = (& pnpm --version); uv = (& uv --version)
        python = (& uv run --project workers/irl python --version)
        dotnet = (& dotnet --version); rust = @(& rustc +1.99.0 -Vv)
        rustCfg = @(& rustc +1.99.0 --print cfg)
        os = @{ caption = $os.Caption; version = $os.Version; build = $os.BuildNumber; architecture = $os.OSArchitecture }
        overrideNamesPresent = @('RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','CARGO_BUILD_TARGET' | Where-Object { Test-Path "Env:$_" })
    } | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $output 'environment.json') -Encoding utf8
    $metadata = & cargo +1.99.0 metadata --locked --format-version 1 --manifest-path reports/P0-06/sqlite-probe/Cargo.toml | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'Cargo metadata failed' }
    @($metadata.packages | Select-Object name,version,license) | ConvertTo-Json |
        Set-Content (Join-Path $output 'probe-dependencies.json') -Encoding utf8
    Check 'probe-fmt' { cargo +1.99.0 fmt --manifest-path reports/P0-06/sqlite-probe/Cargo.toml -- --check }
    Check 'probe-clippy' { cargo +1.99.0 clippy --locked --manifest-path reports/P0-06/sqlite-probe/Cargo.toml -- -D warnings }
    Check 'probe-run' { cargo +1.99.0 run --locked --manifest-path reports/P0-06/sqlite-probe/Cargo.toml }
    Check 'diff-check' { git diff --check }
    $checks.Add([pscustomobject]@{ check = 'five-original-hashes-and-routing'; exitCode = 0 })
} finally {
    $checks | ConvertTo-Json | Set-Content (Join-Path $output 'checks.json') -Encoding utf8
    Write-Output "Evidence: $([IO.Path]::GetRelativePath($root, $output))"
}
