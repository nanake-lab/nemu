# 型号与价格核对 — 2026-11-11（Australia/Perth）

先从进程OPENAI_API_KEY查询GET https://api.openai.com/v1/models，再核对官方页面。可见家族ID为gpt-5.6-sol、gpt-5.6-terra、gpt-6-sol、gpt-6.1-sol、gpt-6.1-terra。第二次同一只读查询用于保存原始响应；没有生成调用。完整列表响应只放私有仓。

| 模型 | Standard输入/输出（US$/百万token） | 缓存读/30分钟写/12小时写 | 决定 |
| --- | --- | --- | --- |
| gpt-6.1-terra | 1 / 5 | 0.05 / 1.25 / 2.50 | 唯一选择；同代较低费用，保留长评分与重发空间 |
| gpt-6.1-sol | 2 / 10 | 0.10 / 2.50 / 5 | 不调用；同用量约两倍费用 |

来源：[官方价格页](https://developers.openai.com/api/docs/pricing)、[Terra模型页](https://developers.openai.com/api/docs/models/gpt-6.1-terra)、[Sol模型页](https://developers.openai.com/api/docs/models/gpt-6.1-sol)。以上来自打开后的正文，模型页和价格页数值一致。Terra支持high和Chat Completions；Snapshots列出gpt-6.1-terra，本密钥列表也有同名ID，未列日期后缀。固定该完整ID，不用latest、不猜日期、不自动升级。公开snapshot名称不是可验证的权重哈希，不承诺底层build永不改变。

采用全球直接API、Standard/default、短上下文；不使用Flex/Batch/Fast、区域端点、工具或缓存优惠。预留输入按最高列示缓存写费率2.50、输出5计；不靠缓存命中省钱。执行前600秒内重新核价，价格/型号/身份变化停止。当前核对不保证未来价格。

原始models响应、时间及预算证据见[EVIDENCE-INDEX](EVIDENCE-INDEX.json)。
