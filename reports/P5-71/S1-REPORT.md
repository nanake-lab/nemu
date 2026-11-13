验证档位：重档；审查后受影响子集补跑与新增分析，原 run02 保留为带缺口研究记录。

# P5-71 S1 补充报告

## 当前状态

补充预注册与静态白名单已先推送（`5f736a91`，停机字段编码勘误 `310b5b62`），随后实现。本文件待本轮执行结束追加结果；现在不签发补充实验通过，也不改原 run02 标签。保持 Draft / needs-user，不合并、不领下一卡。

## 改动与验证边界

- 旧 F 参考按本步 θ 重求原事件 D；测试适配器首个内部记录 n=0，结果 n_issued=1 不改。生产 fast/dynamics 未改。手算及历史自查的修订见 [REFERENCE-SELF-AUDIT](REFERENCE-SELF-AUDIT.md)。
- `full_run.py` 区分自身 I23、跨实现位比较及引用审计；`criteria.distance_failure` 接入扩展非扩张判定。
- C-02 合成 H 保存实际 F 作用视图（b/g/λ、R+、解释/治理内容）与来源位置，并逐步核对 receipt。对象用已有 AuditExtension 的 CanonicalV1 包封绑定；θ 引用标签是测试解析器的合同键，包封不是可导入的生产 ThetaSnapshotV3。Python 独立编码包封并算摘要，不借 DUT 生成期望。不签发生产 H 解析、完整快照恢复或治理验证。
- `s1_analysis.py` 从独立参考传播历史区间、条件于锁定 pull 的背景与基本运算包络；实数性质、浮点点值、舍入包络、未知的超越函数误差分开。原09-18轨迹包含事件前Φ，不能把纯事件的整数枚举结果当作实际轨迹；两种读数分别保存。
- `s1_run.py` 只读静态白名单，拒绝覆盖输出目录；新短见证的分隔前提前置检查，4小时单调时钟限制覆盖正式执行、分析及新见证缩减。旧结果从未续跑或拼接。
- 正常 CI 的 `p571_full_tooling_boundaries` 调用参考手算、S1 分析/门检查和 C-02 修改幅度的软件夹具；这些不算正式研究样本。既有 `full_f.rs` 与新增 `s1_run.py` 超过400行：前者集中公开F适配/历史回执，后者集中单次执行、只读分析和证据生命周期；没有新增生产层或通用框架，分开数值计算的分析模块与引用模块保持单一用途。

| 检查 | 当前结果 | 私有日志位置（归档索引待收尾） |
| --- | --- | --- |
| Cargo release 构建 | 通过，`[optimized]`，47.26秒 | `s1-checks/release-build.log`、编译器artifact JSON |
| `cargo test -p core-kernel --release --locked` | 单元273 passed、1既有ignored（163.01秒）；集成4 passed（4.36秒） | `s1-checks/kernel-test-01.log` |
| release clippy all-targets `-D warnings` | 通过，10.31秒 | `s1-checks/clippy-01.log` |
| `cargo fmt --all --check` | 通过 | `s1-checks/fmt-check-01.log` |
| 锁定 Ruff / mypy strict | 通过，30个Python文件 | `s1-checks/ruff-preflight.log`、`mypy-preflight.log` |
| 手算与工具自查 | 通过；旧断言失败、静态错误及修复日志保留 | `s1-checks/reference-01.log`、`reference-02.log` |

## 基线与未解除边界

最近有效 run02 CI 基线 B=`88be8131d1240209da0a1c9428f2e0875c0e4380`，本次核对 M=`5403d40d`。B 至 M 的差异为文档、报告、代理规则和 docs guard；ADR0035新增发布时点说明，P5-30/76新增未来时间加速实验，未改本卡冻结资源、数值代码或验收定义。无相关构建/运行基线变化，未 merge main。当前 head 的 CI 另列，不把旧 CI 当成新代码绿灯。

原07-11的0/4、原旧F混合参考的失败、123/11/1447缩减边界保持历史结论。本轮尚待独立交叉审查，不自动清除整卡阻塞。固定ρ公开入口、内部候选/等界、生产消费/身份/存储/迁移/事务/真实时间，以及P5-76闭环、P5-77判断质量/载体继续 not_run。通用 C 故障次序与旧严格下溢全域仍未认证。

没有新增依赖、生产参数、契约或用户数据。
