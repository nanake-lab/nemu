P5-22 将终端契约升级到 CONTRACTS v2.2：身体读出 v2、主动边界、成长查询/通知/暂停恢复、载荷摘要绑定审批及社交行动权限。旧消息和字节保留，使用后继消息。

Claude 首轮审查阻塞项已修订：

- B1：validateApproval 在 await 前复制并校验，返回不可变、一次性获批载荷快照；consume 检查新鲜时间，不再用裸布尔值授权其他可变缓冲区。
- B2：新增 core.queryResult.v2 与 queryResultToPanel，完整保留非空 L/证据引用，growth 和普通查询返回数组互斥。
- D1：主动边界的消息、枚举别名及 JSON 投影进入权威 IDL；生成资源类型/浏览器 zod 校验，配置外形和值不变。

用户授权的 ADR 0007 required oneof 保留：u32 选中字段号后接值，缺失/多选/未知号/尾随字节拒绝，不改其他编码规则。P5-01 回溯意见相关的文本门禁和 P5-32 基线已加入。继承的非阻塞 C1(a) 单向 supersession 问题明确留待后续。

验证：

- 合并 origin/main=47ee284 后，本机 26 项命令全通过：全仓 474 测试、contracts 343 测试，Rust fmt/clippy/workspace，三个 worker 静态检查/测试。
- 三语言各 10,702 契约用例；protoc 443 向量；0 stale；check_compat 各基线/current/origin/main 0 违例；schema 回归 8 测试。
- 相对被审查 head16711b0，原 433 合法、448 非法、18 Unicode 前缀完全不变；三语言各 1,777 查询、字节/摘要 0 差异、哈希一致。只追加 10 合法、9 非法，目前 443/457/18。已发布 main 的 396/409/18 证据也保留。
- Ubuntu/Windows CI 以本 PR 最终 head Checks 及修订评论为准；全部验收完成前保持草稿。

报告：reports/P5-22/REPORT.md；本轮 reports/P5-22/REVISION-1.md（含配套修改、哈希、逐项回归）；规格对应 docs/traceability/P5-22.md。

**请 Claude 交叉复审。** 请复审 B1/B2/D1，并一并审查 CONTRACTS、IDL、生成器/浏览器校验器、三语言编解码、黄金字节、兼容门禁及 V2-51–54。实现方不自行给出交叉审查结论。

保留 needs-user：Claude 通过后，由用户批准并 squash 合并。无 GPU、无用户谱系访问、无新增依赖。
