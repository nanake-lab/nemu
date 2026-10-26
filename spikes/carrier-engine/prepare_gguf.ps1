[CmdletBinding()]
param(
    [string]$CacheRoot = 'D:\Dev\cache\P0-10'
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$cache = [IO.Path]::GetFullPath($CacheRoot)
if (-not $cache.StartsWith('D:\Dev\cache\', [StringComparison]::OrdinalIgnoreCase)) {
    throw 'P0-10 artifacts must stay under D:\Dev\cache\.'
}
$revision = 'cdbee75f17c01a7cc42f958dc650907174af0554'
$commit = '620d2225cc436ab0b001e3200e590e3b286ef9ab'
$source = Join-Path $cache 'source\llama.cpp-b11669'
$model = Join-Path $cache ('models\Qwen--Qwen3-4B-Instruct-2507\' + $revision)
$engine = Join-Path $cache 'llama.cpp\b11669'
$cpuEngine = Join-Path $cache 'llama.cpp\b11669-cpu-quantize'
$python = Join-Path $cache 'convert-venv\Scripts\python.exe'
$output = Join-Path $cache ('gguf\Qwen--Qwen3-4B-Instruct-2507\' + $revision)
if ((Test-Path -LiteralPath $output) -or (Test-Path -LiteralPath $cpuEngine)) {
    throw 'Output/CPU tool directory exists; preserve and inspect before rerunning.'
}
$sourceManifestPath = (Get-ChildItem -LiteralPath $source -Filter 'source-inventory-*.json' |
    Sort-Object Name | Select-Object -Last 1).FullName
$sourceManifest = Get-Content -Raw -LiteralPath $sourceManifestPath | ConvertFrom-Json
if ($sourceManifest.commit -ne $commit) { throw 'Converter source commit mismatch.' }
foreach ($entry in $sourceManifest.files) {
    $path = Join-Path $source $entry.path
    if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.sha256) {
        throw ('Converter source hash mismatch: ' + $entry.path)
    }
}
$modelInventoryPath = Join-Path $cache 'downloads\inventories\qwen-20261025T151401049785Z.json'
$modelInventory = Get-Content -Raw -LiteralPath $modelInventoryPath | ConvertFrom-Json
if ($modelInventory.revision -ne $revision) { throw 'Model revision mismatch.' }
foreach ($entry in $modelInventory.files) {
    $path = Join-Path $model $entry.path
    if ((Get-Item -LiteralPath $path).Length -ne $entry.bytes -or
        (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.sha256) {
        throw ('Model file integrity mismatch: ' + $entry.path)
    }
}
# Isolate the quantizer from the CUDA/RPC plugins. DLL search uses executable/CWD.
$extraction = Get-Content -Raw -LiteralPath (Join-Path $cache 'downloads\engine\b11669\extraction-20261025-01.json') | ConvertFrom-Json
$cpuFiles = @($extraction.files | Where-Object {
    ($_.path -like '*.dll' -and $_.path -notmatch '^(ggml-cuda|ggml-rpc|cublas.*|cudart.*)\.dll$') -or
    $_.path -eq 'llama-quantize.exe' -or $_.path -eq 'LICENSE-LLVM-OpenMP'
})
New-Item -ItemType Directory -Path $cpuEngine | Out-Null
foreach ($entry in $cpuFiles) {
    $path = Join-Path $engine $entry.path
    if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.sha256) {
        throw ('Engine file integrity mismatch: ' + $entry.path)
    }
    Copy-Item -LiteralPath $path -Destination (Join-Path $cpuEngine $entry.path)
}
if (Get-ChildItem -LiteralPath $cpuEngine -Filter '*cuda*') { throw 'CUDA DLL in CPU tool directory.' }
New-Item -ItemType Directory -Path $output | Out-Null
$logs = Join-Path $output 'evidence'
New-Item -ItemType Directory -Path $logs | Out-Null
$env:CUDA_VISIBLE_DEVICES = '-1'
$env:HF_HUB_OFFLINE = '1'
$env:TRANSFORMERS_OFFLINE = '1'
$env:HF_HUB_DISABLE_TELEMETRY = '1'
$env:HF_HOME = Join-Path $cache 'huggingface'
$env:TORCH_HOME = Join-Path $cache 'torch'
$env:PYTHONDONTWRITEBYTECODE = '1'
$env:OMP_NUM_THREADS = '4'
$env:MKL_NUM_THREADS = '4'
$env:TEMP = Join-Path $cache 'tmp'
$env:TMP = $env:TEMP
Remove-Item Env:GGML_BACKEND_PATH -ErrorAction SilentlyContinue
$script:steps = [Collections.Generic.List[object]]::new()
function Invoke-Recorded([string]$Name, [string]$Program, [string[]]$Arguments) {
    $started = [DateTimeOffset]::UtcNow.ToString('o')
    $log = Join-Path $logs ($Name + '.log')
    & $Program @Arguments 2>&1 | Tee-Object -FilePath $log
    $code = $LASTEXITCODE
    $script:steps.Add([ordered]@{name=$Name; program=$Program; arguments=$Arguments;
        startedUtc=$started; endedUtc=[DateTimeOffset]::UtcNow.ToString('o'); exitCode=$code; log=$log})
    $script:steps | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $logs 'commands.json') -Encoding utf8
    if ($code -ne 0) { throw ($Name + ' failed; original log and partial file retained.') }
}
$cpuCheck = "import torch; assert torch.version.cuda is None; print('torch', torch.__version__, 'CUDA build', torch.version.cuda)"
Invoke-Recorded 'cpu-environment' $python @('-c', $cpuCheck)
$bf16 = Join-Path $output 'carrier-BF16.gguf'
$bf16Partial = $bf16 + '.partial'
Invoke-Recorded 'convert-bf16' $python @((Join-Path $source 'convert_hf_to_gguf.py'), $model,
    '--outfile', $bf16Partial, '--outtype', 'bf16')
Move-Item -LiteralPath $bf16Partial -Destination $bf16
$results = [Collections.Generic.List[object]]::new()
$results.Add([ordered]@{quantization='BF16';path=$bf16;bytes=(Get-Item -LiteralPath $bf16).Length;
    sha256=(Get-FileHash -LiteralPath $bf16 -Algorithm SHA256).Hash.ToLowerInvariant()})
Push-Location $cpuEngine
try {
    foreach ($quant in @('Q4_K_M', 'Q8_0')) {
        $path = Join-Path $output ('carrier-' + $quant + '.gguf')
        $partial = $path + '.partial'
        Invoke-Recorded ('quantize-' + $quant) (Join-Path $cpuEngine 'llama-quantize.exe') @($bf16, $partial, $quant, '4')
        Move-Item -LiteralPath $partial -Destination $path
        $results.Add([ordered]@{quantization=$quant;path=$path;bytes=(Get-Item -LiteralPath $path).Length;
            sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()})
    }
} finally { Pop-Location }
[ordered]@{status='converted-on-cpu-not-inference-tested';modelRevision=$revision;engineCommit=$commit;
    converterSourceInventory=$sourceManifestPath;modelInventory=$modelInventoryPath;
    cpuOnlyToolFiles=@($cpuFiles.path);threads=4;files=@($results);
    endedUtc=[DateTimeOffset]::UtcNow.ToString('o')} |
    ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $output 'inventory.json') -Encoding utf8
Write-Output ('Conversion inventory: ' + (Join-Path $output 'inventory.json'))
