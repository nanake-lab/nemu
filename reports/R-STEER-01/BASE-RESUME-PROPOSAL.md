# Base首分片续传候选（未批准、未执行）

原方法单次900秒超时：curl退出28，保留869,961,948字节，SHA256为`f146443d3b46eefb12a0ca0d1f63ae69f388ee8d9a501c74e2393bb3faf57fdd`。这不是模型或引擎兼容性结果。元数据、已核验5个文件和失败日志见 [原记录](preparation/base-download-01/raw-filemap.json)；权重/partial留D盘，不提交权重。

推荐先跑不依赖Base的C0，再决定是否给同一官方修订的首分片一次**30分钟上限**续传。下面是可核对的精确候选，不执行，也不自动重试：新建目录复制partial，保留首次失败字节；同一curl、同一官方URL用Range续传。若服务器不支持续传或再次失败就停；完整后须按官方SHA256验证。30分钟仍可能不够，不能把用完预算当完整下载；其余分片/转换/行为测量不由此标完成。

```powershell
$sourcePartial = 'D:/Dev/cache/R-STEER-01/models/Qwen--Qwen3.5-4B-Base/1001bb4d826a52d1f399e183466143f4da7b741b/model.safetensors-00001-of-00002.safetensors.partial'
$resumeRoot = 'D:/Dev/cache/R-STEER-01/base-continue-02'
if (Test-Path -LiteralPath $resumeRoot) { throw 'Preserve existing attempt' }
if ((Get-FileHash -LiteralPath $sourcePartial).Hash.ToLowerInvariant() -ne 'f146443d3b46eefb12a0ca0d1f63ae69f388ee8d9a501c74e2393bb3faf57fdd') { throw 'Original partial changed' }
New-Item -ItemType Directory -Path $resumeRoot | Out-Null
$newPartial = Join-Path $resumeRoot 'first-shard.partial'
Copy-Item -LiteralPath $sourcePartial -Destination $newPartial
& curl.exe --fail --location --silent --show-error --connect-timeout 20 --max-time 1800 --continue-at - --output $newPartial 'https://huggingface.co/Qwen/Qwen3.5-4B-Base/resolve/1001bb4d826a52d1f399e183466143f4da7b741b/model.safetensors-00001-of-00002.safetensors' 2> (Join-Path $resumeRoot 'curl.stderr.log')
$curlExit = $LASTEXITCODE
[ordered]@{exitCode=$curlExit; bytes=(Get-Item -LiteralPath $newPartial).Length; sha256=(Get-FileHash -LiteralPath $newPartial).Hash.ToLowerInvariant(); endedUtc=[DateTimeOffset]::UtcNow.ToString('o')} | ConvertTo-Json | Set-Content -Encoding utf8NoBOM (Join-Path $resumeRoot 'result.json')
if ($curlExit -ne 0) { throw 'Resume failed; preserve partial and stop' }
if ((Get-Item -LiteralPath $newPartial).Length -ne 5329398712 -or (Get-FileHash -LiteralPath $newPartial).Hash.ToLowerInvariant() -ne 'df547074dce70532a0493e5433152bd17a65efb89088cfabc2e7e2371a93d712') { throw 'Official shard hash mismatch' }
```

替代选择：暂缓Base，只完成当前载体的可运行部分，Base相关项保留not_run；或另给更长的固定下载预算。都不换来源、模型、权重或全局网络配置。

需要用户决定的依据是 [AGENTS.md §3第13条](../../AGENTS.md) 的“卡住就停下来问”规则；本卡已保留一次认真尝试的原始失败，不自行开始第二方法。GPU窗口仍按用户本次提示和§9单独安排，续传不占用该安静窗口。
