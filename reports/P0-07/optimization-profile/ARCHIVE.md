
### 原始profiler轨迹归档校验

Luna/max 子代理仅打包日志，2026-10-18 UTC 04:39:38–04:39:43、退出0；使用Python标准库gzip，mtime=0。未压缩trace.json留在本机（由本目录.gitignore忽略），仓库保存可无损解压的trace.json.gz。三项解压后字节数与SHA256均等于原始文件，源文件哈希未变。

| 子目录 | 原始bytes | 原始/解压SHA256 | gzip bytes | gzip SHA256 |
| --- | ---: | --- | ---: | --- |
| qwen4-eager-v1 | 53312249 | c96d89838c7620720f636309be7c22ea55bdee23cd96c5dd6811fd24d8b89d5d | 3053891 | 9479dd990404b3ba733d5e50b0cf2035956b7bad6e0c505ce5d884639548f8e1 |
| qwen4-sdpa-v1 | 59247571 | b4e8f7d45d8135538179884a7b6c3da91172e60f57af075cf5883644ff44b171 | 3437946 | e6035db7902ea104f67900b41da771489afb4164ce3bc93b10f575043d3d425c |
| qwen4-native-rms-v1 | 38866347 | 19bd2d251b81e810963f86cec1b99b30ed74096b86de8342b1adc165a056f639 | 2334572 | 5e4b97b1ea8d87f5490c76af785ade90c464c0adda3b64c0c3e42a47c5bd16ae |
