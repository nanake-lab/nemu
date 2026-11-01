[CmdletBinding()]
param([Parameter(Mandatory)][string]$EvidenceDirectory)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$metadataPath = Join-Path $root 'reports/P0-12/preparation/model-metadata.json'
$evidence = [IO.Path]::GetFullPath($EvidenceDirectory)
if (Test-Path -LiteralPath $evidence) { throw 'Use a new evidence directory; preserve previous attempts.' }
New-Item -ItemType Directory -Path $evidence | Out-Null
$metadata = Get-Content -LiteralPath $metadataPath -Raw | ConvertFrom-Json
$revision = '851bf6e806efd8d0a36b00ddf55e13ccb7b8cd0a'
if ($metadata.id -ne 'Qwen/Qwen3.5-4B' -or $metadata.sha -ne $revision) {
    throw 'Official repository/revision mismatch.'
}
$target = Join-Path 'D:/Dev/cache/P0-12/models/Qwen--Qwen3.5-4B' $revision
New-Item -ItemType Directory -Path $target -Force | Out-Null
$records = [Collections.Generic.List[object]]::new()
try {
    foreach ($entry in $metadata.siblings) {
        $name = [string]$entry.rfilename
        if ($name -match '[/\\:]' -or $name -in @('.', '..')) { throw 'Unexpected upstream file path.' }
        $path = Join-Path $target $name
        $url = "https://huggingface.co/Qwen/Qwen3.5-4B/resolve/$revision/$name"
        $started = [DateTimeOffset]::UtcNow.ToString('o')
        $candidate = $path
        if (-not (Test-Path -LiteralPath $path)) {
            $candidate = $path + '.partial'
            # User authorized network retries. Each curl attempt resumes from actual length;
            # curl -C rejects unsupported/incorrect range responses. Hash still gates acceptance.
            for ($attempt = 1; $attempt -le 12; $attempt++) {
                Write-Output "Downloading $name ($($entry.size) bytes), attempt $attempt"
                & curl.exe --fail --location --silent --show-error --connect-timeout 20 --max-time 1800 --speed-limit 1024 --speed-time 120 --continue-at - --output $candidate $url 2> (Join-Path $evidence ($name + ".attempt-$attempt.stderr.log"))
                $code = $LASTEXITCODE
                [ordered]@{file=$name; attempt=$attempt; exitCode=$code; bytes=$(if (Test-Path $candidate) {(Get-Item $candidate).Length} else {0}); endedUtc=[DateTimeOffset]::UtcNow.ToString('o')} |
                    ConvertTo-Json -Compress | Add-Content (Join-Path $evidence 'attempts.jsonl') -Encoding utf8NoBOM
                if ($code -eq 0) { break }
                if ($code -notin @(5,6,7,18,28,35,52,55,56) -or $attempt -eq 12) { throw "Download failed: $name; curl exit $code; partial retained." }
                # Five seconds avoids hammering the same endpoint during a transient outage.
                Start-Sleep -Seconds 5
            }
        }
        $actualSize = (Get-Item -LiteralPath $candidate).Length
        $sha256 = (Get-FileHash -LiteralPath $candidate -Algorithm SHA256).Hash.ToLowerInvariant()
        $lfs = $entry.PSObject.Properties['lfs']
        if ($actualSize -ne $entry.size) { throw "File size mismatch: $name" }
        if ($null -ne $lfs -and $sha256 -ne $entry.lfs.sha256) { throw "Upstream SHA256 mismatch: $name" }
        if ($null -eq $lfs) {
            $data = [IO.File]::ReadAllBytes($candidate)
            $header = [Text.Encoding]::UTF8.GetBytes("blob $actualSize`0")
            $blob = [Security.Cryptography.SHA1]::HashData([byte[]]($header + $data))
            if ([Convert]::ToHexString($blob).ToLowerInvariant() -ne $entry.blobId) { throw "Git blob mismatch: $name" }
        }
        if ($candidate -ne $path) { Move-Item -LiteralPath $candidate -Destination $path }
        $records.Add([ordered]@{path=$name; bytes=$actualSize; sha256=$sha256; source=$url; startedUtc=$started; endedUtc=[DateTimeOffset]::UtcNow.ToString('o')})
        $records | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $evidence 'download-files.json') -Encoding utf8NoBOM
        Write-Output "Verified $name $sha256"
    }
    [ordered]@{repository=$metadata.id; revision=$revision; target=$target; status='downloaded-and-verified'; files=@($records.ToArray()); metadataSha256=(Get-FileHash $metadataPath).Hash.ToLowerInvariant()} |
        ConvertTo-Json -Depth 6 | Set-Content (Join-Path $evidence 'model-inventory.json') -Encoding utf8NoBOM
} catch {
    [ordered]@{status='failed-stop'; error=$_.Exception.Message; endedUtc=[DateTimeOffset]::UtcNow.ToString('o'); verifiedFiles=$records.Count} |
        ConvertTo-Json | Set-Content (Join-Path $evidence 'download-failure.json') -Encoding utf8NoBOM
    throw
}
