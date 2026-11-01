[CmdletBinding()]
param()
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$evidence = Join-Path $root 'reports/P0-12/preparation'
$metadataPath = Join-Path $evidence 'model-metadata.json'
$metadata = Get-Content -LiteralPath $metadataPath -Raw | ConvertFrom-Json
$revision = '851bf6e806efd8d0a36b00ddf55e13ccb7b8cd0a'
if ($metadata.id -ne 'Qwen/Qwen3.5-4B' -or $metadata.sha -ne $revision) {
    throw 'Official repository/revision mismatch.'
}
$target = Join-Path 'D:/Dev/cache/P0-12/models/Qwen--Qwen3.5-4B' $revision
if (Test-Path -LiteralPath $target) { throw 'Preserve existing model download; no automatic retry.' }
New-Item -ItemType Directory -Path $target | Out-Null
$records = [Collections.Generic.List[object]]::new()
try {
    foreach ($entry in $metadata.siblings) {
        $name = [string]$entry.rfilename
        if ($name -match '[/\\:]' -or $name -in @('.', '..')) { throw 'Unexpected upstream file path.' }
        $path = Join-Path $target $name
        $url = "https://huggingface.co/Qwen/Qwen3.5-4B/resolve/$revision/$name"
        $started = [DateTimeOffset]::UtcNow.ToString('o')
        Write-Output "Downloading $name ($($entry.size) bytes)"
        # Thirty minutes per file and two minutes stalled limit network waits; no retries.
        & curl.exe --fail --location --silent --show-error --connect-timeout 20 --max-time 1800 --speed-limit 1024 --speed-time 120 --output ($path + '.partial') $url
        $code = $LASTEXITCODE
        if ($code -ne 0) { throw "Download failed: $name; curl exit $code; partial retained." }
        $actualSize = (Get-Item -LiteralPath ($path + '.partial')).Length
        $sha256 = (Get-FileHash -LiteralPath ($path + '.partial') -Algorithm SHA256).Hash.ToLowerInvariant()
        $lfs = $entry.PSObject.Properties['lfs']
        if ($actualSize -ne $entry.size) { throw "File size mismatch: $name" }
        if ($null -ne $lfs -and $sha256 -ne $entry.lfs.sha256) { throw "Upstream SHA256 mismatch: $name" }
        if ($null -eq $lfs) {
            $data = [IO.File]::ReadAllBytes($path + '.partial')
            $header = [Text.Encoding]::UTF8.GetBytes("blob $actualSize`0")
            $blob = [Security.Cryptography.SHA1]::HashData([byte[]]($header + $data))
            if ([Convert]::ToHexString($blob).ToLowerInvariant() -ne $entry.blobId) { throw "Git blob mismatch: $name" }
        }
        Move-Item -LiteralPath ($path + '.partial') -Destination $path
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
