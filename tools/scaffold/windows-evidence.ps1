param(
    [Parameter(Mandatory)][ValidateSet('Init', 'Snapshot', 'Run', 'Verify')][string]$Mode,
    [ValidateSet('transport', 'isolation', 'e2e', 'visual')][string]$Check,
    [ValidatePattern('^[a-z-]+$')][string]$Label = 'snapshot'
)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false

if ($Mode -eq 'Init') {
    if (!$env:GITHUB_RUN_ID -or !$env:GITHUB_RUN_ATTEMPT -or !$env:RUNNER_TEMP) {
        throw 'Run identity and RUNNER_TEMP are required (use explicit local IDs outside CI).'
    }
    $root = Join-Path $env:RUNNER_TEMP "nemu-evidence/$env:GITHUB_RUN_ID/$env:GITHUB_RUN_ATTEMPT/windows"
    if (Test-Path -LiteralPath $root) { throw "Evidence directory already exists: $root" }
    $started = [DateTime]::UtcNow
    New-Item -ItemType Directory -Path $root | Out-Null
    @{
        run = $env:GITHUB_RUN_ID; attempt = $env:GITHUB_RUN_ATTEMPT
        sha = (git rev-parse HEAD); startedUtc = $started.ToString('o')
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $root 'run.json') -Encoding utf8
    $env:NEMU_EVIDENCE_DIR = $root
    if ($env:GITHUB_ENV) { "NEMU_EVIDENCE_DIR=$root" >> $env:GITHUB_ENV }
    Write-Output $root
    exit 0
}

$root = $env:NEMU_EVIDENCE_DIR
if (!$root -or !(Test-Path -LiteralPath (Join-Path $root 'run.json'))) { throw 'Missing evidence initialization' }

function Save-Snapshot([string]$Name) {
    $sample = [ordered]@{ at = [DateTime]::UtcNow.ToString('o'); interactive = [Environment]::UserInteractive
        sessionId = (Get-Process -Id $PID).SessionId }
    # No command lines, environment dumps, window titles, or document content.
    $sample.processes = @(Get-Process | Select-Object Id, ProcessName, SessionId)
    try { $sample.ports = @(Get-NetTCPConnection -State Listen -ErrorAction Stop |
        Select-Object LocalAddress, LocalPort, OwningProcess) } catch { $sample.portsError = $_.Exception.Message }
    $query = Join-Path $env:SystemRoot 'System32/query.exe'
    if (Test-Path -LiteralPath $query) {
        # Session listing includes state (Active/Disc), not application content.
        $sample.sessions = @(& $query session 2>&1 | ForEach-Object { "$_" })
        $sample.sessionQueryExit = $LASTEXITCODE
    } else { $sample.sessionsError = 'query.exe unavailable' }
    $sample.lockState = 'Unknown: interactive/session Active does not prove unlocked'
    try {
        $since = [DateTime]::Parse((Get-Content -LiteralPath (Join-Path $root 'run.json') -Raw | ConvertFrom-Json).startedUtc)
        $sample.sessionEvents = @(Get-WinEvent -FilterHashtable @{
            LogName = 'Microsoft-Windows-TerminalServices-LocalSessionManager/Operational'; StartTime = $since
        } -ErrorAction Stop | Select-Object TimeCreated, Id)
    } catch { $sample.sessionEventsError = $_.Exception.Message }
    $sample | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $root "$Name.json") -Encoding utf8
}

if ($Mode -eq 'Snapshot') { Save-Snapshot $Label; exit 0 }

if ($Mode -eq 'Run') {
    if (!$Check) { throw 'Check required' }
    Save-Snapshot "$Check-before"
    $started = [DateTime]::UtcNow.ToString('o')
    $previousDebug = $env:DEBUG
    $code = 1
    try {
        # Playwright captures child stdout AND stderr from spawn, including before launch resolves.
        if ($Check -eq 'e2e') { $env:DEBUG = 'pw:browser' }
        $log = Join-Path $root "$Check.log"
        if ($Check -eq 'isolation') {
            & pwsh -NoProfile -File (Join-Path $PSScriptRoot 'test-windows-isolation.ps1') 2>&1 | Tee-Object -FilePath $log
        } else {
            & pnpm "test:$Check" 2>&1 | Tee-Object -FilePath $log
        }
        $code = $LASTEXITCODE
    } finally {
        $env:DEBUG = $previousDebug
        if ($Check -eq 'visual') {
            # P1-01 owns this producer and still writes its legacy location. Import only files
            # freshly written by this invocation, never the historical report directory.
            $source = Join-Path (Get-Location) 'reports/P1-01/visual'
            $destination = Join-Path $root 'stage-visual'
            New-Item -ItemType Directory -Path $destination | Out-Null
            if (Test-Path -LiteralPath $source) {
                Get-ChildItem -LiteralPath $source -File | Where-Object {
                    $_.LastWriteTimeUtc -ge [DateTime]::Parse($started).ToUniversalTime()
                } | ForEach-Object {
                    if ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Visual evidence link forbidden' }
                    Copy-Item -LiteralPath $_.FullName -Destination $destination
                }
            }
            if ($code -eq 0 -and !(Test-Path -LiteralPath (Join-Path $destination 'comparison.json'))) {
                throw 'Successful visual check did not generate current-run comparison.json'
            }
        }
        if ($Check -eq 'e2e' -and (Test-Path -LiteralPath $log)) {
            foreach ($stream in @('out', 'err')) {
                $lines = @(Get-Content -LiteralPath $log | Where-Object { $_ -match "\[pid=\d+\]\[$stream\]" })
                # Preserve markers/timing; empty stream is explicitly represented by an empty file.
                [IO.File]::WriteAllLines((Join-Path $root "electron-$stream.log"), [string[]]$lines)
            }
        }
        Save-Snapshot "$Check-after"
        @{ check = $Check; startedUtc = $started; endedUtc = [DateTime]::UtcNow.ToString('o'); exitCode = $code } |
            ConvertTo-Json | Set-Content -LiteralPath (Join-Path $root "$Check-result.json") -Encoding utf8
    }
    exit $code
}

# A fresh, absent-before-init directory is the provenance boundary. Never copy old reports here.
$identity = Get-Content -LiteralPath (Join-Path $root 'run.json') -Raw | ConvertFrom-Json
if ($identity.run -ne $env:GITHUB_RUN_ID -or $identity.attempt -ne $env:GITHUB_RUN_ATTEMPT -or
    $identity.sha -ne (git rev-parse HEAD)) { throw 'Evidence identity mismatch' }
$start = [DateTime]::Parse($identity.startedUtc).ToUniversalTime()
$end = [DateTime]::UtcNow
if (Get-ChildItem -LiteralPath $root -Recurse -Attributes ReparsePoint) { throw 'Evidence links forbidden' }
$manifest = @(Get-ChildItem -LiteralPath $root -Recurse -File | Where-Object {
    $_.FullName -ne (Join-Path $root 'manifest.json')
} | ForEach-Object {
    if ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Evidence links forbidden' }
    if ($_.CreationTimeUtc -lt $start -or $_.LastWriteTimeUtc -lt $start -or $_.LastWriteTimeUtc -gt $end) {
        throw "Not generated in this attempt: $($_.Name)"
    }
    @{ path = [IO.Path]::GetRelativePath($root, $_.FullName); bytes = $_.Length
       sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash
       createdUtc = $_.CreationTimeUtc.ToString('o'); writtenUtc = $_.LastWriteTimeUtc.ToString('o') }
})
@{ identity = $identity; verifiedUtc = $end.ToString('o'); files = $manifest } |
    ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $root 'manifest.json') -Encoding utf8
Write-Output "Verified $($manifest.Count) current-attempt evidence files"
