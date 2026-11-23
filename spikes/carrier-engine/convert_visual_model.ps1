[CmdletBinding()]
param()
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$evidence = Join-Path $root 'reports/P0-12/preparation'
$source = 'D:/Dev/cache/P0-10/source/llama.cpp-b11669'
$python = 'D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe'
$cpuEngine = 'D:/Dev/cache/P0-10/llama.cpp/b11669-cpu-quantize'
$inventory = Get-Content (Join-Path $evidence 'model-inventory.json') -Raw | ConvertFrom-Json
$output = 'D:/Dev/cache/P0-12/gguf'
if (Test-Path -LiteralPath $output) { throw 'Output exists; preserve previous attempt.' }
$model = $inventory.target
foreach ($entry in $inventory.files) {
    if ((Get-FileHash -LiteralPath (Join-Path $model $entry.path)).Hash.ToLowerInvariant() -ne $entry.sha256) {
        throw "Model hash mismatch: $($entry.path)"
    }
}
$sourceInventory = Get-Content (Join-Path $source 'source-inventory-20261025T171354189657Z.json') -Raw | ConvertFrom-Json
if ($sourceInventory.commit -ne '620d2225cc436ab0b001e3200e590e3b286ef9ab') { throw 'Wrong converter commit.' }
foreach ($entry in $sourceInventory.files) {
    if ((Get-FileHash -LiteralPath (Join-Path $source $entry.path)).Hash.ToLowerInvariant() -ne $entry.sha256) { throw "Converter changed: $($entry.path)" }
}
$extraction = Get-Content 'D:/Dev/cache/P0-10/downloads/engine/b11669/extraction-20261025-01.json' -Raw | ConvertFrom-Json
foreach ($file in Get-ChildItem -LiteralPath $cpuEngine -File) {
    $expected = @($extraction.files | Where-Object { $_.path -eq $file.Name })
    if ($expected.Count -ne 1 -or (Get-FileHash $file.FullName).Hash.ToLowerInvariant() -ne $expected[0].sha256) { throw "CPU engine hash mismatch: $($file.Name)" }
}
if (Get-ChildItem -LiteralPath $cpuEngine -Filter '*cuda*') { throw 'CUDA plugin in CPU quantizer directory.' }
New-Item -ItemType Directory -Path $output | Out-Null
New-Item -ItemType Directory -Path 'D:/Dev/cache/P0-12/tmp' -Force | Out-Null
$env:CUDA_VISIBLE_DEVICES = '-1'
$env:HF_HUB_OFFLINE = '1'
$env:TRANSFORMERS_OFFLINE = '1'
$env:HF_HUB_DISABLE_TELEMETRY = '1'
$env:PYTHONDONTWRITEBYTECODE = '1'
$env:HF_HOME = 'D:/Dev/cache/P0-12/huggingface'
$env:TEMP = 'D:/Dev/cache/P0-12/tmp'
$env:TMP = $env:TEMP
$env:OMP_NUM_THREADS = '4'
$env:MKL_NUM_THREADS = '4'
Remove-Item Env:GGML_BACKEND_PATH -ErrorAction SilentlyContinue
$script:steps = [Collections.Generic.List[object]]::new()
function Invoke-Step([string]$Name, [string]$Program, [string[]]$Arguments) {
    $started = [DateTimeOffset]::UtcNow.ToString('o')
    & $Program @Arguments *> (Join-Path $evidence ($Name + '.log'))
    $code = $LASTEXITCODE
    $script:steps.Add([ordered]@{name=$Name; program=$Program; arguments=$Arguments; startedUtc=$started; endedUtc=[DateTimeOffset]::UtcNow.ToString('o'); exitCode=$code})
    $script:steps | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $evidence 'conversion-commands.json') -Encoding utf8NoBOM
    if ($code -ne 0) { throw "$Name failed ($code); stop, preserve logs and partial files." }
}
Invoke-Step 'cpu-environment' $python @('-c', 'import torch; assert torch.version.cuda is None; print(torch.__version__)')
$bf16 = Join-Path $output 'carrier-BF16.gguf'
# The main 32-layer decoder is measured; no speculative MTP draft is used.
Invoke-Step 'convert-bf16' $python @((Join-Path $source 'convert_hf_to_gguf.py'), $model, '--outfile', $bf16, '--outtype', 'bf16', '--no-mtp')
Push-Location $cpuEngine
try {
    Invoke-Step 'quantize-Q4_K_M' (Join-Path $cpuEngine 'llama-quantize.exe') @($bf16, (Join-Path $output 'carrier-Q4_K_M.gguf'), 'Q4_K_M', '4')
} finally { Pop-Location }
# F16 keeps the visual encoder unquantized, using the pinned converter's mmproj path.
Invoke-Step 'convert-mmproj' $python @((Join-Path $source 'convert_hf_to_gguf.py'), $model, '--mmproj', '--outfile', (Join-Path $output 'mmproj-F16.gguf'), '--outtype', 'f16')
[ordered]@{revision=$inventory.revision; engineCommit=$sourceInventory.commit; files=@(Get-ChildItem $output -Filter '*.gguf' | ForEach-Object { [ordered]@{path=$_.FullName; bytes=$_.Length; sha256=(Get-FileHash $_.FullName).Hash.ToLowerInvariant()} })} |
    ConvertTo-Json -Depth 5 | Set-Content (Join-Path $evidence 'gguf-inventory.json') -Encoding utf8NoBOM
