param([string]$Repository = 'D:\Dev\nemu-wt\P5-04')
$ErrorActionPreference = 'Stop'
$evidenceDir = $PSScriptRoot
$sourceHead = 'fbcb7cc14ee492893729833237bf955cc57462b6'
$runId = 40323615588
$expectedDigest = '20c648f230b27a177cf8182d3488e35cd7e07d93a4c6db065c9cc6b8f5de1dec'
$connector = Get-Content -LiteralPath (Join-Path $evidenceDir 'connector-evidence.json') -Raw | ConvertFrom-Json
if ($connector.head -ne $sourceHead -or $connector.run_id -ne $runId -or
    $connector.artifact.workflow_run.head_sha -ne $sourceHead) { throw 'Provenance mismatch' }
$archivePath = Join-Path $evidenceDir 'windows-P5-02-original.zip'
$archiveDigest = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($archiveDigest -ne $expectedDigest -or $connector.artifact.digest -ne "sha256:$archiveDigest") {
    throw 'Archive digest mismatch'
}
$zip = [IO.Compression.ZipFile]::OpenRead($archivePath)
try {
    $names = @($zip.Entries.FullName | Sort-Object)
    if (($names -join ',') -ne 'clippy.log,toolchain.log') { throw 'Unexpected archive contents' }
    foreach ($entry in $zip.Entries) {
        $stream = $entry.Open()
        $buffer = [IO.MemoryStream]::new()
        try {
            $stream.CopyTo($buffer)
            $extracted = [IO.File]::ReadAllBytes((Join-Path $evidenceDir "artifact/$($entry.FullName)"))
            if ([Convert]::ToBase64String($buffer.ToArray()) -cne [Convert]::ToBase64String($extracted)) {
                throw "Extracted bytes differ: $($entry.FullName)"
            }
        } finally { $stream.Dispose(); $buffer.Dispose() }
    }
} finally { $zip.Dispose() }
foreach ($job in $connector.jobs) {
    $savedBytes = [IO.File]::ReadAllBytes((Join-Path $evidenceDir "job-$($job.id).log"))
    $decodedBytes = [Text.UTF8Encoding]::new($false).GetBytes($job.log)
    if ([Convert]::ToBase64String($savedBytes) -cne [Convert]::ToBase64String($decodedBytes)) {
        throw "Decoded job log differs: $($job.id)"
    }
}
$steps = (Get-Content -LiteralPath (Join-Path $evidenceDir 'job-steps.json') -Raw | ConvertFrom-Json).steps
if (($steps | Where-Object number -eq 19).conclusion -ne 'failure') { throw 'Step 19 mismatch' }
if (@($steps | Where-Object { $_.number -ge 4 -and $_.number -le 18 -and $_.conclusion -eq 'skipped' }).Count -ne 15) {
    throw 'Skipped steps mismatch'
}
if (Test-Path -LiteralPath (Join-Path $evidenceDir 'artifact/tests.log')) { throw 'Unexpected tests.log' }
$clippyText = Get-Content -LiteralPath (Join-Path $evidenceDir 'artifact/clippy.log') -Raw
if ($clippyText -notmatch 'download of pp/v-/ppv-lite86 failed' -or
    $clippyText -notmatch 'transferred 0 bytes') { throw 'Failure evidence missing' }
$workflowBlob = git -C $Repository rev-parse 'HEAD:.github/workflows/windows.yml'
$mainWorkflowBlob = git -C $Repository rev-parse 'origin/main:.github/workflows/windows.yml'
if ($workflowBlob -ne $mainWorkflowBlob -or $workflowBlob -ne '65b29f2a8a73a7bb08c44492646f3952d8a17435') {
    throw 'Windows workflow differs from expected main'
}
git -C $Repository diff --quiet $sourceHead HEAD -- core apps packages workers executors tools services config profiles content .github package.json pnpm-lock.yaml pnpm-workspace.yaml
if ($LASTEXITCODE -ne 0) { throw 'Runtime or workflow changes since diagnosed head' }
$head = git -C $Repository rev-parse HEAD
$main = git -C $Repository rev-parse origin/main
git -C $Repository merge-base --is-ancestor $main HEAD
if ($LASTEXITCODE -ne 0) { throw 'Latest fetched main not merged' }
$result = [ordered]@{
    checked_at_utc = [DateTime]::UtcNow.ToString('o'); result = 'passed'; source_head = $sourceHead
    inspected_local_head = $head; merged_main = $main; run_id = $runId; job_id = 121574544222
    artifact_id = $connector.artifact.id; artifact_sha256 = $archiveDigest; extracted_bytes_identical = $true
    decoded_logs_identical = $true; missing_tests_log = $true; desktop_steps_skipped = 15
    workflow_blob = $workflowBlob; runtime_and_workflow_changes_since_source_head = 0
    ci_retried = $false; compilation_or_tests_rerun = $false; system_or_acl_changes = $false
}
$result | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $evidenceDir 'verification.json') -Encoding utf8
$hashes = Get-ChildItem -LiteralPath $evidenceDir -File -Recurse | Where-Object Name -ne 'SHA256.json' |
    Sort-Object FullName | ForEach-Object {
        [ordered]@{ path = [IO.Path]::GetRelativePath($evidenceDir, $_.FullName).Replace('\','/'); bytes = $_.Length
            sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant() }
    }
$hashes | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $evidenceDir 'SHA256.json') -Encoding utf8
$result | ConvertTo-Json -Depth 4
