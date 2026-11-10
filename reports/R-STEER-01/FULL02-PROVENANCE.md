# FULL-02采集谱系与复算入口

最终504条来自同一DS-RUBRIC-02和同一批冻结合成材料，501格式/逐字引证合规、3invalid、0最终技术失败。两轮20小样、FULL-01七次仅作方法材料，评分不进入正式总体；材料重合已事先披露，不冒充新留出集。每个条目有唯一最终结果，72个反序重复不是独立场景。

| 执行段与实际head | 新HTTP尝试 | 本段结果及保留的停止 | 累计预算上界US$ | 原始证据 |
| --- | ---: | --- | ---: | --- |
| FULL-02初段144e6b1a | 70 | 70分；终端不断弹窗，用户反馈后user_stop，在途正常收尾；创建后隐藏仍闪的失败保留 | 0.290715432 | [原70](ds-flash-full-02/raw-filemap.json) |
| 无窗口0c66e05d | 41 | 40分+1次IncompleteRead未知；合计110分，按当时网络规则停；未保留partial/读前HTTP状态，缺口不回补 | 0.403365240 | [原111及原件](ds-flash-full-02-windowless-v2/raw-filemap.json) |
| 有限重发73c96dd2 | 156 | 155分+1完整length空评分；旧缺失项只重发一次成功，整额预留永久保留；完整length误归段级停止，原fatal不改 | 0.850617732 | [完整响应及账](ds-flash-full-02-network-retry-02/raw-filemap.json) |
| 完整invalid分类修复2319988a | 238 | 236分+2invalid；携入265分和1完整invalid，最终501+3；原完整空响应不重发、不增8192预算 | 1.483366224 | [完整最终语料/账/源](ds-flash-full-02-complete-invalid-01/raw-filemap.json) |

FULL-02共505次尝试，其中504份完整HTTP响应、1次历史读取未知；加此前方法材料48次（47完整/1未知），预算账553次记录、551完整、2未知。未知项分别US$0.011795700、0.011862300整额保留；成功重发另按US$0.011862300整额保留，不能将它也称未知。按公开峰值费率计算的是保守预算上界，非账户账单、不含支付税费，没有查余额或账户。

原[止于七次的FULL-01](ds-flash-full-01/raw-filemap.json)不倒改，也不复用其中六份合法分数。两轮小样见[方法材料](ds-flash-high-review-02/REVIEW.md)。旧high主管失去终态的原因仍未知，旧预留不销账。官方Base下载/curl28、curl56/CPU转换记录错误与成功均见[交接/转换](BASE-HANDOFF-03.md)，Base行为未运行。

## 冻结与执行分类

- [FULL-02预注册](DS-FULL-02.md)：仅在valence后加一句枚举，原rubric不改；完整schema/引证非法留作invalid/原分母失败继续。
- [无窗口封装](DS-FULL-02-WINDOWLESS.md)：不改请求，只在启动处禁创建窗口；未运行修订1的计划与源快照留存。离线真实三次句柄0，实际观察一次四子进程句柄0，非持续监控。
- [有限网络重发](FULL02-NETWORK-RETRY-01.md)：用户明确授权，只对不完整且无可解析评分内容最多重发2次、至少30秒，完整响应一律不重发；真实loopback间隔30.188秒。修订1仅准备，修订2补账本IO失败立即全局停止；请求/阈值不变。
- [完整空评分分类修复](FULL02-COMPLETE-INVALID-01.md)：按既有完整invalid单项继续授权修正封装，将完整非stop评分记invalid；原fatal保留，不增加token/重发旧评分。
- [最终52源/504请求/265+1携入/238工作项计划](inputs/ds-full02-complete-invalid-plan-01.json)；[源/完整请求快照](ds-length-preparation-01/raw-filemap.json)。前段46源/110分/394项与52源链均可逐字节复算。

所有方向、层/强度、混合、文字状态干预、厂商人格/Base行为及投影仍not_run。过程封装变化不签发C1测量可用性、方向有效或非零m批准。零注入原始C0/C1的题/分/阈值/停止结果不变。

## 独立机械证据

[FULL-02初始冻结](ds-full2-audit-01/raw-filemap.json)、[无窗口冻结](ds-full2-windowless-audit-01/raw-filemap.json)、[无窗口终态](ds-full2-windowless-final-audit-01/raw-filemap.json)、[有限重发冻结](ds-retry-freeze-audit-01/raw-filemap.json)、[有限重发终态](ds-retry-final-audit-01/raw-filemap.json)、[最终分类续段冻结](ds-length-freeze-audit-01/raw-filemap.json)。机械代理按配置gpt-6.1-terra/max显式参数试用，不声称角色名自动加载；只核技术/字节/费用，不判断语义可信或正式门。

检查首轮Ruff/mypy的导入/类型/格式错误均保留，修正后窄复核通过；新增HTTP503正文取证和账本IO停止只跑对应新增测试，未重跑已通过的30秒测试。首轮九项测试AST逐方法未变。[网络检查与源快照](ds-retry-checks-01/raw-filemap.json)、[修正/503检查](ds-retry-checks-02/raw-filemap.json)、[IO停止检查](ds-retry-checks-03/raw-filemap.json)、[原fixture](ds-retry-offline-fixtures-01/raw-filemap.json)、[完整length单项invalid检查](ds-length-checks-01/raw-filemap.json)。展示程序曾因默认GBK读取UTF-8失败，部分产物/失败程序保留，后继新目录仅修编码，未重跑API。

KEY仅一次性不回显交接到进程环境，不入研究日志、参数、文件或仓库；用户直接给出后不再要求手工输入。模型API实际身份/指纹/UTC和官方价格快照随各原始目录保存；官方API无不可变权重快照，不能把文档版本或同指纹当权重哈希。

## 用户抽查与正式使用

[固定随机20并列页](ds-full02-review-01/REVIEW.md) · [全部3个invalid](ds-full02-review-01/INVALID.md) · [原20 ID/抽样方法/数据SHA](ds-full02-review-01/sample-index.json)。同seed20261110、全504均匀无放回，不按质量/语言换样；3个invalid都未抽中，另完整附出。完整评分、对象、逐字证据、事实参照和理由不改。

全504数据清单SHA256为`8e331da96f76feba9a0f0c3773f7d1c3b251f6a3c4b5df74efed672870bcf648`。尚无用户对新20的确认，sample-review.json/formal-analysis.json不存在；不会用旧小样确认代签。独立样本与终态审计只核字节和计费，不计算C1正式门。用户认为评分不可信则停，不调口径重试。
