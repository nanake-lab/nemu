# R-JUDGE-01 — OpenAI 第二盲评员校准方案

本轮只冻结方案：用一个OpenAI模型对已有合成材料独立评分，检查复述状态漏判和invalid是否重现。尚未评分，尚未校准通过；本轮仅查询 /v1/models。

选择 gpt-6.1-terra、high、16384 completion tokens，最多4在途、每次180秒、累计主动90分钟，费用硬线US$25。477条历史用量换算情景约US$15.62，但全部用满输出的保守预留约US$50.72，不能保证25美元跑完；逐次预留，到线停止，未跑项保留原分母。

- [冻结方案](PREREG-01.md)：模型、材料、盲法、终点、重发和停止规则。
- [证据索引](EVIDENCE-INDEX.json)：私有来源、477份离线请求模板、模型列表查询证据、预算核算。
- [核价摘要](SOURCES.md)：2026-11-11官方来源。

编号核对：开工前fetch；origin/main、本地/远端分支、GitHub issue/PR搜索未见R-JUDGE-01已认领。搜索返回的PR #116是R-LIFE-01，不构成该号认领。本卡PR #161，分支task/R-JUDGE-01-openai-calibration，从74f7bcaf41ec45bb31063fc079eeeae49927ac82创建独立工作树。

范围仅reports/R-JUDGE-01/与私有R-JUDGE-01/。R-STEER-01来源固定私有提交933c8509bbe7954ddb35f2fa1cae03c7d0475d9c，公开说明参考其分支7977e4148b1233e6103b389ba21e1cd12ee42c4f；未改其文件，也未改核心、契约、规律、正式资源或cards.json。

验证：477份离线模板及JSON解析、prompt字符串对照、来源/模板SHA绑定、git diff --check与本次Markdown相对链接检查。全工作区测试、Windows/GPU及评分API均not_run；新增运行时依赖零。离线准备曾因Windows默认GBK读取UTF-8失败，改为按原始UTF-8字节解析后完成；未触碰源字节。

待用户确认（阻塞执行）：审查方案，尤其C2旧口径与J v1输出上限的可比性限制、预算可能提前停止、模型固定ID的公开snapshot粒度。保持draft/needs-user，写「请审查方案」后停止，不合并、不领取下一卡。
