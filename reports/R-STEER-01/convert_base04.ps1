[CmdletBinding()]
param()
# CPU-only preparation. Run in a child pwsh; environment changes are process-local.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$cache = 'D:/Dev/cache/R-STEER-01'
$model = Join-Path $cache 'base-download-03/model'
$evidence = Join-Path $cache 'base-convert-04'
$output = Join-Path $cache 'base-gguf-04'
$revision = '1001bb4d826a52d1f399e183466143f4da7b741b'
$source = 'D:/Dev/cache/P0-10/source/llama.cpp-b11669'
$python = 'D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe'
$cpuEngine = 'D:/Dev/cache/P0-10/llama.cpp/b11669-cpu-quantize'
if ((Test-Path -LiteralPath $evidence) -or (Test-Path -LiteralPath $output)) { throw 'Existing attempt; preserve it, do not retry automatically' }
if ((Get-PSDrive D).Free -lt 40000000000) { throw '40 GB preparation reserve required' }
New-Item -ItemType Directory -Path $evidence | Out-Null
$script:steps = [Collections.Generic.List[object]]::new()
$script:verified = [Collections.Generic.List[object]]::new()
function Test-File([string]$Path, [string]$Expected, [long]$Bytes = -1) {
    $item = Get-Item -LiteralPath $Path
    $sha = (Get-FileHash -LiteralPath $Path).Hash.ToLowerInvariant()
    if ($sha -ne $Expected -or ($Bytes -ge 0 -and $item.Length -ne $Bytes)) { throw "Hash/size mismatch: $Path" }
    $script:verified.Add([ordered]@{path=$item.FullName; bytes=$item.Length; sha256=$sha})
}
function Invoke-Step([string]$Name, [string]$Program, [string[]]$Arguments) {
    $started = [DateTimeOffset]::UtcNow.ToString('o')
    & $Program @Arguments *> (Join-Path $evidence ($Name + '.log'))
    $code = $LASTEXITCODE
    $script:steps.Add([ordered]@{name=$Name; program=$Program; arguments=$Arguments; workingDirectory=(Get-Location).Path; startedUtc=$started; endedUtc=[DateTimeOffset]::UtcNow.ToString('o'); exitCode=$code})
    $script:steps | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $evidence 'commands.json') -Encoding utf8NoBOM
    if ($code -ne 0) { throw "$Name failed ($code); preserve logs/partial and stop" }
}
try {
    $inventoryPath = Join-Path $cache 'base-download-03/model-inventory.json'
    $inventory = Get-Content -Raw $inventoryPath | ConvertFrom-Json
    $completion = Get-Content -Raw (Join-Path $cache 'base-download-03/result.json') | ConvertFrom-Json
    if ($inventory.repository -ne 'Qwen/Qwen3.5-4B-Base' -or $inventory.revision -ne $revision -or $inventory.license -ne 'Apache-2.0') { throw 'Wrong inventory identity' }
    if ([IO.Path]::GetFullPath($inventory.target) -ne [IO.Path]::GetFullPath($model)) { throw 'Wrong model path: no legacy cache fallback' }
    $actualPaths = @(Get-ChildItem -LiteralPath $model -File | ForEach-Object Name | Sort-Object)
    $expectedPaths = @($inventory.files.path | Sort-Object)
    if ($expectedPaths.Count -ne 13 -or @(Compare-Object $actualPaths $expectedPaths).Count -ne 0) { throw 'Unexpected file set' }
    if ($completion.status -ne 'complete' -or $completion.bytes -ne 9342824751 -or $completion.verifiedFiles -ne 13) { throw 'Missing complete handoff' }
    foreach ($entry in $inventory.files) {
        if ($entry.path -match '[/\\:]' -or $entry.path -in @('.', '..')) { throw 'Unexpected model path' }
        Test-File (Join-Path $model $entry.path) $entry.sha256 $entry.bytes
    }
    $sourceInventoryPath = Join-Path $source 'source-inventory-20261025T171354189657Z.json'
    $sourceInventory = Get-Content -Raw $sourceInventoryPath | ConvertFrom-Json
    if ($sourceInventory.commit -ne '620d2225cc436ab0b001e3200e590e3b286ef9ab') { throw 'Wrong converter commit' }
    foreach ($entry in $sourceInventory.files) { Test-File (Join-Path $source $entry.path) $entry.sha256 }
    $extractionPath = 'D:/Dev/cache/P0-10/downloads/engine/b11669/extraction-20261025-01.json'
    $extraction = Get-Content -Raw $extractionPath | ConvertFrom-Json
    foreach ($file in Get-ChildItem -LiteralPath $cpuEngine -File) {
        $expected = @($extraction.files | Where-Object { $_.path -eq $file.Name })
        if ($expected.Count -ne 1) { throw "Unknown CPU engine file: $($file.Name)" }
        Test-File $file.FullName $expected[0].sha256
    }
    if (Get-ChildItem -LiteralPath $cpuEngine -Filter '*cuda*') { throw 'CUDA plugin in CPU quantizer directory' }
    foreach ($path in @($PSCommandPath,$python,$inventoryPath,$sourceInventoryPath,$extractionPath)) {
        $item=Get-Item -LiteralPath $path
        $script:verified.Add([ordered]@{path=$item.FullName; bytes=$item.Length; sha256=(Get-FileHash -LiteralPath $path).Hash.ToLowerInvariant()})
    }
    $script:verified | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $evidence 'verified-inputs.json') -Encoding utf8NoBOM
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
    [ordered]@{CUDA_VISIBLE_DEVICES=$env:CUDA_VISIBLE_DEVICES; HF_HUB_OFFLINE=$env:HF_HUB_OFFLINE; TRANSFORMERS_OFFLINE=$env:TRANSFORMERS_OFFLINE; HF_HUB_DISABLE_TELEMETRY=$env:HF_HUB_DISABLE_TELEMETRY; PYTHONDONTWRITEBYTECODE=$env:PYTHONDONTWRITEBYTECODE; HF_HOME=$env:HF_HOME; TEMP=$env:TEMP; TMP=$env:TMP; OMP_NUM_THREADS=$env:OMP_NUM_THREADS; MKL_NUM_THREADS=$env:MKL_NUM_THREADS; GGML_BACKEND_PATH_removed=$true; pwsh=$PSVersionTable.PSVersion.ToString()} | ConvertTo-Json | Set-Content (Join-Path $evidence 'process-environment.json') -Encoding utf8NoBOM
    # gguf is vendored under the hashed converter source, not a pip distribution.
    Invoke-Step 'cpu-environment' $python @('-c','import sys,torch,transformers; assert torch.version.cuda is None; print(sys.version); print(torch.__version__); print(transformers.__version__)')
    $bf16=Join-Path $output 'carrier-BF16.gguf'
    Invoke-Step 'convert-bf16' $python @((Join-Path $source 'convert_hf_to_gguf.py'),$model,'--outfile',$bf16,'--outtype','bf16','--no-mtp')
    Push-Location $cpuEngine
    try { Invoke-Step 'quantize-Q4_K_M' (Join-Path $cpuEngine 'llama-quantize.exe') @($bf16,(Join-Path $output 'carrier-Q4_K_M.gguf'),'Q4_K_M','4') } finally { Pop-Location }
    Invoke-Step 'convert-mmproj' $python @((Join-Path $source 'convert_hf_to_gguf.py'),$model,'--mmproj','--outfile',(Join-Path $output 'mmproj-F16.gguf'),'--outtype','f16')
    [ordered]@{revision=$revision; engineCommit=$sourceInventory.commit; files=@(Get-ChildItem -LiteralPath $output -Filter '*.gguf' | ForEach-Object { [ordered]@{path=$_.FullName; bytes=$_.Length; sha256=(Get-FileHash -LiteralPath $_.FullName).Hash.ToLowerInvariant()} })} | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $evidence 'gguf-inventory.json') -Encoding utf8NoBOM
    [ordered]@{status='complete'; inference='not_run'; endedUtc=[DateTimeOffset]::UtcNow.ToString('o')} | ConvertTo-Json | Set-Content (Join-Path $evidence 'result.json') -Encoding utf8NoBOM
} catch {
    [ordered]@{status='failed'; message=$_.Exception.Message; detail=$_.ToString(); endedUtc=[DateTimeOffset]::UtcNow.ToString('o')} | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $evidence 'failure.json') -Encoding utf8NoBOM
    throw
}
