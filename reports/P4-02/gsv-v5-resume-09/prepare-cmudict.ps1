$ErrorActionPreference='Stop'
$d='D:\Dev\cache\p4-02\gsv-v5-resume-09'
$rev='550b6625bcef1f2abff2ff770a5a0d272c9c6b2a'
$target='D:\Dev\cache\p4-02\nltk_data\corpora'
New-Item -ItemType Directory -Force $target | Out-Null
foreach($name in @('cmudict')){
 foreach($ext in @('xml','zip')){
  $file=if($ext -eq 'zip'){Join-Path $target "$name.zip"}else{Join-Path $d "$name.xml"}
  for($i=1;$i -le 5;$i++){
   curl.exe --location --fail --continue-at - --retry 0 --connect-timeout 30 --output $file "https://raw.githubusercontent.com/nltk/nltk_data/$rev/packages/corpora/$name.$ext" *> "$d\$name-$ext-attempt-$i.log"
   $e=$LASTEXITCODE; Set-Content "$d\$name-$ext-attempt-$i.exit.txt" $e
   if($e -eq 0){break}; if($e -notin @(5,6,7,18,28,35,52,55,56) -or $i -eq 5){exit $e}; Start-Sleep -Seconds 30
  }
  Get-FileHash $file -Algorithm SHA256 | ConvertTo-Json -Compress | Add-Content "$d\nltk-hashes.jsonl"
 }
 Expand-Archive -LiteralPath (Join-Path $target "$name.zip") -DestinationPath $target
}
