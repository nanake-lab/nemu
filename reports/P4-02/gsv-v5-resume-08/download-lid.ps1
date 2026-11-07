$d='D:\Dev\cache\p4-02\gsv-v5-resume-08'
$target='D:\Dev\cache\p4-02\gpt-sovits-v5-official\GPT_SoVITS\pretrained_models\fast_langdetect\lid.176.bin'
New-Item -ItemType Directory -Force (Split-Path $target) | Out-Null
for($i=1;$i -le 5;$i++){
 curl.exe --location --fail --continue-at - --retry 0 --connect-timeout 30 --output $target 'https://dl.fbaipublicfiles.com/fasttext/supervised-models/lid.176.bin' *> "$d\lid-bin-attempt-$i.log"
 $e=$LASTEXITCODE; Set-Content "$d\lid-bin-attempt-$i.exit.txt" $e
 if($e -eq 0){Get-FileHash $target -Algorithm SHA256 | ConvertTo-Json | Set-Content "$d\lid-bin-sha256.json"; exit 0}
 if($e -notin @(5,6,7,18,28,35,52,55,56) -or $i -eq 5){exit $e}
 Start-Sleep -Seconds 30
}
