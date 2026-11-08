param([Parameter(Mandatory)][ValidateSet('Download','Convert')][string]$Stage)
# Preparation only. No model inference, GPU loading, tool installation or global settings.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$cache = 'D:/Dev/cache/R-STEER-01'
$revision = '1001bb4d826a52d1f399e183466143f4da7b741b'
$model = Join-Path $cache "models/Qwen--Qwen3.5-4B-Base/$revision"
$evidence = Join-Path $cache ('base-' + $Stage.ToLowerInvariant() + '-01')
if (Test-Path -LiteralPath $evidence) { throw 'Preserve previous evidence; do not retry automatically' }
if ((Get-PSDrive D).Free -lt 40000000000) { throw '40 GB preparation reserve required' }
New-Item -ItemType Directory -Path $evidence | Out-Null
if ($Stage -eq 'Download') {
    if (Test-Path -LiteralPath $model) { throw 'Model directory exists; preserve it' }
    New-Item -ItemType Directory -Path $model | Out-Null
    $metadata = Invoke-RestMethod "https://huggingface.co/api/models/Qwen/Qwen3.5-4B-Base/revision/$revision`?blobs=true"
    $metadata | ConvertTo-Json -Depth 30 | Set-Content (Join-Path $evidence 'metadata.json') -Encoding utf8NoBOM
    if ($metadata.id -ne 'Qwen/Qwen3.5-4B-Base' -or $metadata.sha -ne $revision -or $metadata.cardData.license -ne 'apache-2.0') { throw 'Official metadata mismatch' }
    $records = [Collections.Generic.List[object]]::new()
    try {
        foreach ($entry in $metadata.siblings) {
            $name = [string]$entry.rfilename
            if ($name -match '[/\\:]' -or $name -in @('.', '..')) { throw 'Unexpected upstream path' }
            $path = Join-Path $model $name
            $partial = $path + '.partial'
            $url = "https://huggingface.co/Qwen/Qwen3.5-4B-Base/resolve/$revision/$name"
            $started = [DateTimeOffset]::UtcNow.ToString('o')
            Write-Output "Downloading $name ($($entry.size) bytes)"
            & curl.exe --fail --location --silent --show-error --connect-timeout 20 --max-time 900 --output $partial $url 2> (Join-Path $evidence ($name + '.stderr.log'))
            $code = $LASTEXITCODE
            if ($code -ne 0) { throw "curl exit $code on $name; partial retained, no retry" }
            $bytes = (Get-Item -LiteralPath $partial).Length
            $sha = (Get-FileHash -LiteralPath $partial).Hash.ToLowerInvariant()
            if ($bytes -ne $entry.size) { throw "Wrong bytes: $name" }
            if ($null -ne $entry.PSObject.Properties['lfs']) {
                if ($sha -ne $entry.lfs.sha256) { throw "Upstream SHA256 mismatch: $name" }
            } else {
                $header = [Text.Encoding]::UTF8.GetBytes("blob $bytes`0")
                $gitBlob = [Security.Cryptography.SHA1]::HashData([byte[]]($header + [IO.File]::ReadAllBytes($partial)))
                if ([Convert]::ToHexString($gitBlob).ToLowerInvariant() -ne $entry.blobId) { throw "Git blob mismatch: $name" }
            }
            Move-Item -LiteralPath $partial -Destination $path
            $records.Add([ordered]@{path=$name; bytes=$bytes; sha256=$sha; source=$url; startedUtc=$started; endedUtc=[DateTimeOffset]::UtcNow.ToString('o')})
            $records | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $evidence 'download-files.json') -Encoding utf8NoBOM
            Write-Output "Verified $name $sha"
        }
        [ordered]@{repository=$metadata.id; revision=$revision; license='Apache-2.0'; target=$model; files=@($records.ToArray())} | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $evidence 'model-inventory.json') -Encoding utf8NoBOM
    } catch {
        [ordered]@{error=$_.Exception.Message; verifiedFiles=$records.Count; endedUtc=[DateTimeOffset]::UtcNow.ToString('o')} | ConvertTo-Json | Set-Content (Join-Path $evidence 'failure.json') -Encoding utf8NoBOM
        throw
    }
    exit 0
}
$source = 'D:/Dev/cache/P0-10/source/llama.cpp-b11669'
$python = 'D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe'
$cpuEngine = 'D:/Dev/cache/P0-10/llama.cpp/b11669-cpu-quantize'
$inventory = Get-Content -Raw (Join-Path $cache 'base-download-01/model-inventory.json') | ConvertFrom-Json
if ($inventory.revision -ne $revision) { throw 'Wrong downloaded revision' }
foreach ($entry in $inventory.files) {
    if ((Get-FileHash -LiteralPath (Join-Path $model $entry.path)).Hash.ToLowerInvariant() -ne $entry.sha256) { throw "Changed model file: $($entry.path)" }
}
$sourceInventory = Get-Content -Raw (Join-Path $source 'source-inventory-20261025T171354189657Z.json') | ConvertFrom-Json
if ($sourceInventory.commit -ne '620d2225cc436ab0b001e3200e590e3b286ef9ab') { throw 'Wrong converter commit' }
foreach ($entry in $sourceInventory.files) {
    if ((Get-FileHash -LiteralPath (Join-Path $source $entry.path)).Hash.ToLowerInvariant() -ne $entry.sha256) { throw "Changed converter: $($entry.path)" }
}
$extraction = Get-Content -Raw 'D:/Dev/cache/P0-10/downloads/engine/b11669/extraction-20261025-01.json' | ConvertFrom-Json
foreach ($file in Get-ChildItem -LiteralPath $cpuEngine -File) {
    $expected = @($extraction.files | Where-Object { $_.path -eq $file.Name })
    if ($expected.Count -ne 1 -or (Get-FileHash $file.FullName).Hash.ToLowerInvariant() -ne $expected[0].sha256) { throw "Changed CPU engine: $($file.Name)" }
}
if (Get-ChildItem -LiteralPath $cpuEngine -Filter '*cuda*') { throw 'CUDA plugin in CPU quantizer directory' }
$output = Join-Path $cache 'base-gguf-01'
if (Test-Path -LiteralPath $output) { throw 'Preserve previous GGUF' }
New-Item -ItemType Directory -Path $output | Out-Null
$taskTemp = Join-Path $evidence 'tmp'
New-Item -ItemType Directory -Path $taskTemp | Out-Null
$env:CUDA_VISIBLE_DEVICES='-1'
$env:HF_HUB_OFFLINE='1'
$env:TRANSFORMERS_OFFLINE='1'
$env:HF_HUB_DISABLE_TELEMETRY='1'
$env:PYTHONDONTWRITEBYTECODE='1'
$env:HF_HOME=Join-Path $cache 'huggingface'
$env:TEMP=$taskTemp
$env:TMP=$taskTemp
$env:OMP_NUM_THREADS='4'
$env:MKL_NUM_THREADS='4'
Remove-Item Env:GGML_BACKEND_PATH -ErrorAction SilentlyContinue
$script:steps = [Collections.Generic.List[object]]::new()
function Invoke-Step([string]$Name, [string]$Program, [string[]]$Arguments) {
    $started = [DateTimeOffset]::UtcNow.ToString('o')
    & $Program @Arguments *> (Join-Path $evidence ($Name + '.log'))
    $code = $LASTEXITCODE
    $script:steps.Add([ordered]@{name=$Name; program=$Program; arguments=$Arguments; startedUtc=$started; endedUtc=[DateTimeOffset]::UtcNow.ToString('o'); exitCode=$code})
    $script:steps | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $evidence 'commands.json') -Encoding utf8NoBOM
    if ($code -ne 0) { throw "$Name failed ($code); preserve evidence and stop" }
}
Invoke-Step 'cpu-environment' $python @('-c','import torch; assert torch.version.cuda is None; print(torch.__version__)')
$bf16=Join-Path $output 'carrier-BF16.gguf'
Invoke-Step 'convert-bf16' $python @((Join-Path $source 'convert_hf_to_gguf.py'),$model,'--outfile',$bf16,'--outtype','bf16','--no-mtp')
Push-Location $cpuEngine
try { Invoke-Step 'quantize-Q4_K_M' (Join-Path $cpuEngine 'llama-quantize.exe') @($bf16,(Join-Path $output 'carrier-Q4_K_M.gguf'),'Q4_K_M','4') } finally { Pop-Location }
Invoke-Step 'convert-mmproj' $python @((Join-Path $source 'convert_hf_to_gguf.py'),$model,'--mmproj','--outfile',(Join-Path $output 'mmproj-F16.gguf'),'--outtype','f16')
[ordered]@{revision=$revision; engineCommit=$sourceInventory.commit; files=@(Get-ChildItem $output -Filter '*.gguf' | ForEach-Object { [ordered]@{path=$_.FullName; bytes=$_.Length; sha256=(Get-FileHash $_.FullName).Hash.ToLowerInvariant()} })} | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $evidence 'gguf-inventory.json') -Encoding utf8NoBOM
