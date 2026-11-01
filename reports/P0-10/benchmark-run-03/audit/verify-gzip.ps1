param([Parameter(Mandatory)][string]$Root, [Parameter(Mandatory)][string]$OutputPath)
$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $OutputPath) { throw 'Preserve previous verification' }
$startedUtc = [DateTime]::UtcNow.ToString('o')
$mapPath = Join-Path $Root 'raw-filemap.json'
$map = Get-Content -LiteralPath $mapPath -Raw | ConvertFrom-Json -AsHashtable
$verified = @{}
foreach ($item in $map.objects.GetEnumerator()) {
    $path = Join-Path $Root $item.Value.path
    $packed = [IO.File]::ReadAllBytes($path)
    $packedHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($packed)).ToLowerInvariant()
    if ($packed.LongLength -ne $item.Value.bytes -or $packedHash -ne $item.Value.sha256) { throw "Packed mismatch: $path" }
    $inputStream = [IO.MemoryStream]::new($packed, $false)
    $gzip = [IO.Compression.GZipStream]::new($inputStream, [IO.Compression.CompressionMode]::Decompress)
    $outputStream = [IO.MemoryStream]::new()
    try { $gzip.CopyTo($outputStream); $restored = $outputStream.ToArray() }
    finally { $gzip.Dispose(); $inputStream.Dispose(); $outputStream.Dispose() }
    $rawHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($restored)).ToLowerInvariant()
    if ($rawHash -ne $item.Key) { throw "Restored hash mismatch: $path" }
    $verified[$item.Value.path] = @{ sha256 = $rawHash; bytes = $restored.LongLength }
}
$originalBytes = 0L
foreach ($file in $map.files) {
    $object = $verified[$file.object]
    if ($null -eq $object -or $object.sha256 -ne $file.sha256 -or $object.bytes -ne $file.bytes) { throw "Mapping mismatch: $($file.path)" }
    $originalBytes += $file.bytes
}
if ($map.files.Count -ne $map.fileCount -or $originalBytes -ne $map.originalBytes -or $verified.Count -ne $map.uniqueFiles) { throw 'Aggregate mismatch' }
$result = [ordered]@{
    startedUtc = $startedUtc; endedUtc = [DateTime]::UtcNow.ToString('o')
    method = '.NET GZipStream + SHA256, independent of Python archiver'
    methodSha256 = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant()
    mapSha256 = (Get-FileHash -LiteralPath $mapPath -Algorithm SHA256).Hash.ToLowerInvariant()
    filesVerified = $map.files.Count; objectsVerified = $verified.Count
    originalBytes = $originalBytes; packedBytes = $map.packedBytes; mismatches = 0
}
$result | ConvertTo-Json | Set-Content -LiteralPath $OutputPath -Encoding utf8NoBOM
$result | ConvertTo-Json -Compress
