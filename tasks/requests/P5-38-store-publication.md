# P5-38 存储发布与读端兼容范围

卡面的两个 cognition 子目录覆盖不了权威事务写入。原有存储每次提交都消费一条 D，恢复和索引也假设每次提交有一个 EventEnvelope。独处时需要在同一个 D 游标上发布多次，继续使用唯一写者、原子可见性和持久回执。

## 用户在草稿 PR #238 施工期间确认的范围

| 位置 | 依据 | 接口与限制 |
| --- | --- | --- |
| core-store、core-runtime/src/store.rs | ADR 0046 §4，首次范围确认 | 写方从事件 V2 直接切换到 V4，新活动经 V4.activity_commit 发布。旧标签按旧格式读取。迁移、回退、混合历史、审计与故障均用临时库。 |
| core-types 与 core/contracts/templates 下的 latest_step、origin_record | ADR 0046 §4，首次范围确认 | 两处引用接受旧活动记录或新活动记录。重新生成 Python、TypeScript 产物，保留旧向量正反例和字节。PR 使用「契约变更」标签。 |
| gateway/src/progress.rs | 第二次范围确认 | 只改终端「已提交」回执的快照读点。V2 事件和 V4 事件可继续核验已观察到的提交；内部活动与未知标签不能产生回执。 |
| core-store 原始字节资源 | ADR 0046 §2.1，第二次范围确认 | 独立类型，普通 SHA-256 与原始长度，沿用对象预算。应用层请求体排除 HTTP 头与凭证。作为文字使用时严格校验 UTF-8 与区间两端。 |
| fact-view 的快照身份比较和同类读端 | 第三次范围确认 | 混合历史按实际标签核验；memory.rs、source.rs 从存储的事件查询取数据。材料由运行时发起查询，向任务层传结果、W19 原标记与来源。 |
| core-runtime/Cargo.toml、core-store/Cargo.toml、core/Cargo.lock | 最终依赖确认 | 只加入 core-runtime → task-layer 和 core-store → 已锁定的 sha2 0.11.1。task-layer 运行时只依赖 core-types。 |

运行时通过注入的查询函数调用 W19。工作区中尚无同时拥有运行时和 W19 的产品调用方。临时库组合验证、交付限制和剩余验收见 [报告](../../reports/P5-38/REPORT.md)。
