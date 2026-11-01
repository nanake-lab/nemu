param([string]$Destination='D:\Dev\models\multilingual-e5-small\614241f622f53c4eeff9890bdc4f31cfecc418b3')
$ErrorActionPreference='Stop'
$manifest=Get-Content -LiteralPath (Join-Path $PSScriptRoot '..\models\manifest.json') -Raw | ConvertFrom-Json
if($manifest.license -ne 'MIT') {throw 'Model license requires explicit user approval'}
if($IsWindows -and [IO.Path]::GetPathRoot([IO.Path]::GetFullPath($Destination)) -ne 'D:\') {throw 'P5-04 model files must be on D:'}
New-Item -ItemType Directory -Force $Destination | Out-Null
foreach($asset in $manifest.files) {
 if($asset.status -eq 'rejected_candidate') {Write-Output "$($asset.path): rejected candidate, not downloaded"; continue}
 if($asset.path -ne [IO.Path]::GetFileName($asset.path)) {throw 'Invalid manifest filename'}
 if(-not $asset.source.StartsWith("https://huggingface.co/intfloat/multilingual-e5-small/resolve/$($manifest.revision)/")) {throw 'Non-official or unpinned model source'}
 $target=Join-Path $Destination $asset.path
 $valid=(Test-Path -LiteralPath $target) -and ((Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant() -eq $asset.sha256)
 if(-not $valid) {Invoke-WebRequest $asset.source -OutFile $target -TimeoutSec 600}
 if((Get-Item -LiteralPath $target).Length -ne $asset.bytes -or (Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant() -ne $asset.sha256) {throw "Integrity failure: $($asset.path)"}
 Write-Output "$($asset.path): SHA-256 verified"
}
