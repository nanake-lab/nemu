param([string]$Repository = 'D:\Dev\nemu-wt\P5-04')
$ErrorActionPreference = 'Stop'
$workflow = Get-Content -LiteralPath (Join-Path $Repository '.github/workflows/ci.yml') -Raw
$scopeStart = $workflow.IndexOf('      - name: Decide scope')
$scopeEnd = $workflow.IndexOf('      - uses: actions/setup-node', $scopeStart)
$scopeBlock = $workflow.Substring($scopeStart, $scopeEnd - $scopeStart)
$scriptText = ($scopeBlock -split '\r?\n' | Select-Object -Skip 10 |
    ForEach-Object { $_ -replace '^          ', '' }) -join "`n"
if ($scriptText -notmatch '\$full = \$true' -or $scriptText -notmatch 'GITHUB_OUTPUT') {
    throw 'Scope extraction failed'
}
$scopeScript = [scriptblock]::Create($scriptText)
$cpuStep = ($workflow -split '      - name: W19 optional CPU runtime compile checks')[1] -split '      - name:' | Select-Object -First 1
if ($cpuStep -notmatch "(?m)^        if: steps\.scope\.outputs\.rust == 'true'\r?$") {
    throw 'W19 CPU check does not use the Rust scope gate'
}
$cases = @(
    @{name='stage-only'; event='pull_request'; paths=@('apps/stage/src/main.ts'); rust=$false; storage=$false},
    @{name='desktop-only'; event='pull_request'; paths=@('apps/desktop/src/main.ts'); rust=$false; storage=$false},
    @{name='fact-view'; event='pull_request'; paths=@('core/crates/fact-view/src/paging.rs'); rust=$true; storage=$false},
    @{name='cargo-manifest'; event='pull_request'; paths=@('core/Cargo.toml'); rust=$true; storage=$true},
    @{name='cargo-lock'; event='pull_request'; paths=@('core/Cargo.lock'); rust=$true; storage=$true},
    @{name='rust-toolchain'; event='pull_request'; paths=@('core/rust-toolchain.toml'); rust=$true; storage=$true},
    @{name='workflow'; event='pull_request'; paths=@('.github/workflows/ci.yml'); rust=$true; storage=$true},
    @{name='shared-root'; event='pull_request'; paths=@('package.json'); rust=$true; storage=$true},
    @{name='manual-full'; event='workflow_dispatch'; paths=@(); rust=$true; storage=$true},
    @{name='diff-unavailable'; event='pull_request'; paths=@(); diffExit=1; rust=$true; storage=$true},
    @{name='docs-only'; event='pull_request'; paths=@('reports/P5-04/REPORT.md'); rust=$false; storage=$false}
)
$taskTemp = Join-Path 'D:\Dev\cache\tmp' ('P5-04-scope-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $taskTemp | Out-Null
$envKeys = @('EVENT','BEFORE','PR_BASE','PR_HEAD','GITHUB_OUTPUT','GITHUB_STEP_SUMMARY')
$savedEnv = @{}
foreach ($key in $envKeys) { $savedEnv[$key] = [Environment]::GetEnvironmentVariable($key, 'Process') }
$results = @()
try {
    foreach ($case in $cases) {
        $script:changedPaths = $case.paths
        $script:diffExit = if ($case.ContainsKey('diffExit')) { $case.diffExit } else { 0 }
        # Only the git diff result is mocked; execute the workflow's actual scope body.
        function git { $global:LASTEXITCODE = $script:diffExit; $script:changedPaths }
        $env:EVENT = $case.event
        $env:BEFORE = 'scope-fixture-base'
        $env:PR_BASE = 'scope-fixture-base'
        $env:PR_HEAD = 'scope-fixture-head'
        $env:GITHUB_OUTPUT = Join-Path $taskTemp ($case.name + '-output.txt')
        $env:GITHUB_STEP_SUMMARY = Join-Path $taskTemp ($case.name + '-summary.txt')
        $output = @(& $scopeScript)
        $actual = @{}
        Get-Content -LiteralPath $env:GITHUB_OUTPUT | ForEach-Object {
            $key, $value = $_ -split '=', 2
            $actual[$key] = $value
        }
        foreach ($key in @('rust','storage')) {
            if ($actual[$key] -cne $case[$key].ToString().ToLowerInvariant()) {
                throw ($case.name + ': wrong ' + $key + ' scope')
            }
        }
        $results += [ordered]@{ name=$case.name; event=$case.event; paths=$case.paths; scope=$actual; output=$output; result='passed' }
    }
} finally {
    foreach ($key in $envKeys) { [Environment]::SetEnvironmentVariable($key, $savedEnv[$key], 'Process') }
    Remove-Item -LiteralPath Function:git
}
[ordered]@{checked_at_utc=[DateTime]::UtcNow.ToString('o'); result='passed'; mock_boundary='git diff only'; cases=$results} |
    ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'ci-scope-result.json') -Encoding utf8
$results | ForEach-Object { $_.name + ': ' + $_.result }
