# 证据索引

私有仓 `nanake-lab/nemu-evidence`，分支 `task/R-STEER-PAIR-01`，固定提交 [`66d60e3c0c9be4c53c9b5e5a8dc44005c966fbaf`](https://github.com/nanake-lab/nemu-evidence/tree/66d60e3c0c9be4c53c9b5e5a8dc44005c966fbaf/R-STEER-PAIR-01)。

[逐文件索引](EVIDENCE-INDEX.json)列出43个文件的私有路径、字节数和SHA-256，共448,414字节。每个实际Git blob与本机文件逐字节相同；原有R-STEER-01证据未改。公开的[承诺](COMMITMENT.json)单独绑定答案表、密文和HTML。

| 私有相对目录 | 内容 |
| --- | --- |
| `R-STEER-PAIR-01/prepared-v1/` | 匿名HTML、带盐答案表的AES-256-GCM密文及公开承诺 |
| `R-STEER-PAIR-01/source/` | 准备/页面/验证源码与预注册快照 |
| `R-STEER-PAIR-01/browser-smoke-01/` | 合成夹具的正常作答、弃权、下载与截图 |
| `R-STEER-PAIR-01/browser-timer-01/` | 合成夹具真实840.175秒截止、保存、截图及计时摘要 |
| `R-STEER-PAIR-01/checks/` | 本地检查日志、实际交付首页验证、专用CI日志、Windows草稿门失败和本地PyYAML缺失记录 |

实际HTML的本机交付位置为 `D:\Dev\cache\R-STEER-PAIR-01\delivery-v1\comparison.html`。目录内另有使用说明。实际首页截图没有题目内容；没有替用户勾选批准或答题。

密钥与种子保存在独立本机custody，不进入这43个文件，不公布其内容或入口链接。人类判断、解盲、结果分析全部 **not_run**；文件名含 `synthetic-result` 的均为技术夹具，不能用作研究结果。

专用CI日志绑定实际checkout `2e9c3a167bd8ef4cd0c1596b08c071d262f4a5b2`，已核实双亲是main `f39f5e9feb4bce7ed56774645b8166e483e7ce7c`、源码head `fdfddd93f5e5cf14bee57614fa135b3b2efb201c`。后续报告提交不冒充这个checkout；合并/最终CI状态另外核验，本轮不合并。
