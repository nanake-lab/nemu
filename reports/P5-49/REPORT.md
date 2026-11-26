重档。交付线 P5-49 3/3 涉及正式 IDL、核心契约校验及 SYS21。

本次冻结后，每份 IRL 候选都可以同当次委派、原始输出、原文和核心阶段采纳记录核对。缺少中性输入或隔离依据的候选只能保留。三个语言实现读取同一组合成数据，编码、摘要和拒收结果一致。契约变更待独立交叉审查及用户合并。

## 契约与兼容

主线基线为 #270 的合并提交 ecee0b0ef5131a09de95ef15cc16a00706519a8d。沿用 #264 批准的 ADR 0051、V2-193 和 ADR 0046–0048。冻结前复核 main 无增量，其余八个开放 PR 与本次文件交集为空，含 #273、#274。

新增 IrlServiceResponse、IrlRemoteAttempt、IrlProducer、IrlCandidateReturn、IrlAdoptionLink 五个内部标签。独立对象使现有生产者继续按旧标签读写。候选、实际输出和采纳仍使用各自的对象引用，保存的正式绑定可供提交方落盘。按次借阅继续只在 fetch 核验成功后扣账。实现、依据与验收入口见[规格对照](../../docs/traceability/P5-49.md)。

## SYS21

| 实际运行的入口 | 观察结果 |
| --- | --- |
| generate.py 与 --check、check_compat.py --ref origin/main | 0 份生成物过期，255 个消息和 93 个枚举，0 项兼容违规。17 份 IDL 及四组向量一起冻结。 |
| freeze_vectors.py | Rust 逐项复验原 906 个合法和 1,385 个非法向量，新增 15 个合法、18 个非法。旧 ingress 63 项、Unicode 18 项也逐项比对一致。 |
| pnpm test:core-contracts | 14 项 schema 测试通过。官方 protoc 与 Protobuf 运行库复验 921 个合法向量。Rust、Python、TypeScript 各 22,762 例，接受 5,147，拒绝 17,615。 |
| 三端同一组定向探针 | 每端 223 例，包括 220 个 IRL 编解码及上传入口对照。成功的上传控制帧逐一换成 13 个内部标签，完整对象入口传合法字节、摘要及长度，均拒收。另向真实解码器提交未知 schema 版本、完整 NatureVector 的 binary64 NaN、FastStateSnapshot 的重复元素计数 0xffffffff，三端分别拒收。 |
| Rust 入口 | cargo test -p core-types --locked 通过 58 项。覆盖两种生产者、不同实请及实返名称、错误原始响应、错次、重复返回、缺失 EV6 依据、截断和陈旧事务。cargo clippy -p core-types --all-targets --locked -- -D warnings 通过。 |
| 源码检查 | Rust 格式检查、夹具 Ruff 与 strict mypy 均通过。首次直接运行 mypy 缺少生成包搜索路径，按仓库 CI 设置 MYPYPATH 后通过。 |

## 限制与交接

真正的 Windows 管道双端认证、SYS02、实际来源注入和录制回放为 not_run，归 P5-07。此处的生产者及中性输入均为合成材料，宿主必须从受信调用独立取得原始请求、响应、解析结果和隔离证据。外层事务原子发布以及 U6 回复时序由运行时接线验证。跨包全工作区 Rust、TS、Python 检查留给托管 CI，本次未运行用户主机的完整 Windows 工作流。

普通 git diff --check 将冻结副本 growth.proto 原有的末尾空行列为一项提示。源文件和副本的 Git blob 均为 d8323f33e7f04b4381d058b977ad15b2e5a1d51e，按既有冻结流程用 git diff --find-copies-harder --check 检查通过。

## 原始证据

[私有证据包](https://github.com/nanake-lab/nemu-evidence/blob/6749f4f19bf3671f250d7d9327fca609e271a41c/P5-49/SYS21/freeze-86145e265f3e.zip) 保存了正式复验的 93 批探针请求与实际返回，另含 223 例三端定向对照、protoc 输出、65 个真实测试状态前缀及其 260 份返回、Rust 测试和静态检查日志。归档大小为 7,866,531 字节，SHA-256 为 `86145e265f3e13f116faaca212d9165345d380bc0026340e435f42704683b15c`。已用远端提交中的 blob 核对上传内容。[SYS21 机器可读结果](SYS21/result.json)
