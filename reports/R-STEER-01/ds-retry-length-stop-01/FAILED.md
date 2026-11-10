# 完整HTTP响应的空评分：不是网络失败、不是随机20

原输出：
```text
复查表明猫恢复良好，同事终于安心了。
```

评分正文为空；finish_reason=length，completion8192全部用于reasoning。HTTP200/response_complete=true，子进程exit0、无墙钟超时。没有合法评分，不重发，不增token。

原封装先检查finish_reason而将它归入段级process_or_integrity_failure；原终态/响应/费用保留。新续段按用户已批准的完整非法单项处理，在新目录记invalid/原分母失败，仅续未发238；旧265分不重评。

[完整原响应](response.json) · [原请求](request.json) · [HTTP终态](transport.json) · [进程终态](worker-terminal.json) · [原分类记录](fatal.json)
