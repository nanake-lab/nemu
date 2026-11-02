$ErrorActionPreference='Stop'
$d='D:\Dev\cache\p4-02\gsv-v5-resume-06'
$root='D:\Dev\cache\p4-02\gpt-sovits-v5-official\GPT_SoVITS\pretrained_models'
$plan=Get-Content D:\Dev\cache\p4-02\gsv-v5-preparation\download-plan.json -Raw | ConvertFrom-Json
foreach($f in ($plan.files | Select-Object -Last 3)) {
 $path=Join-Path $root $f.path
 for($attempt=1;$attempt -le 19;$attempt++) {
  $log=Join-Path $d ((Split-Path $f.path -Leaf)+".attempt-$attempt.log")
  curl.exe --location --fail --continue-at - --retry 0 --connect-timeout 30 --output $path "https://huggingface.co/lj1995/GPT-SoVITS/resolve/$($plan.revision)/$($f.path)" *> $log
  $e=$LASTEXITCODE
  Set-Content "$log.exit.txt" $e
  if($e -eq 0){break}
  if($e -notin @(5,6,7,18,28,35,52,55,56) -or $attempt -eq 19){exit $e}
  Start-Sleep -Seconds 30
 }
 $size=(Get-Item $path).Length
 $hash=(Get-FileHash $path -Algorithm SHA256).Hash.ToLower()
 @{path=$f.path;size=$size;sha256=$hash;expectedSize=$f.size;expectedHash=$f.lfs.oid} | ConvertTo-Json -Compress | Add-Content "$d\verification.jsonl"
 if($size -ne $f.size -or $hash -ne $f.lfs.oid){exit 90}
}
