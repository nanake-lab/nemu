P5-22 终端契约变更：身体读出 v2、主动边界、成长查询/通知/控制、实际载荷审批与社交权限。旧消息、已发布编码和摘要保留，使用后继消息。

本轮修复复审阻塞 A1：模板不再使用会跳过 __proto__ 的 z.record。字典的 JSON 自有条目逐项经过 IDL 的键/值校验，再用 Object.fromEntries 安全重建；合法键无损保留，非法值明确报错，未新增特殊键禁用规则。已重新生成。

回归覆盖 JSON.parse 创建的 __proto__、constructor、prototype、toString、Unicode 键及 null-prototype 字典；检查解析前后键集合、完整 wire 载荷、规范字节和摘要。-1、null、invalid、Infinity、NaN 明确拒绝。新增两个合法黄金向量，旧 443 合法、457 非法、18 Unicode 向量不变。

工作区：D:\Dev\nemu-wt\P5-22；缓存/临时目录使用 D 盘。本轮已合并最新 origin/main=876cff7（包含原先落后的两个提交），合并后重跑全部检查。证据见 reports/P5-22/REVISION-2.md、revision-2-checks/ 和 revision-2-frozen-byte-comparison.json。

最新 26 项检查均有成功记录：全仓 481 测试、contracts 350 测试、三语言各 10,766 用例、protoc 445 向量、0 stale、compat 各基线/current/origin/main 0 违例。迁移导致的 Python 启动器失效已重建环境并通过整组 14 项重跑，原失败日志保留。旧向量三语言各 1,816 查询，字节与摘要 0 差异；两个新增 A1 向量也单列三语言 0 差异证据。

首轮 B1/B2/D1 来源问题已由复审关闭；C1(a) 保持已披露的非阻塞后续。本轮不改 IDL、编解码规则、权限范围或候选配置值，无新增依赖，无 GPU、无用户正式/排练数据访问。

需要独立会话审查；由用户转交。保留 needs-user，执行者不合并。请审查。
