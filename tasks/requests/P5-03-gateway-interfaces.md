# P5-03 / W18 接口请求（2026-10-23）

基线：origin/main 876cff7，P5-02 / PR #58 已合并；P5-22 / PR #57 尚未合并。
P5-03 的授权代码目录只有 `core/crates/gateway/`。本请求不授权修改其他卡的目录，也不修改契约。

## 1. core-store：D 的服务端分配与恢复

现有 `Store::ingest(EventEnvelope)` 要求 `raw_log_position == count(raw_ingress)`，但公开的
`head()` 只给已处理位置，`next_raw()` 只给第一条未处理项。已有多条未处理项时，网关重启不能
从公开 API 得知下一个可追加位置。不能依赖客户端提供位置、扫描 receipt 私有目录、打开第二个
SQLite 连接或另建计数存储。

期望由 core-store 的拥有卡追加一个经过恢复验证的 D 尾位置 API，或在现有单写者内分配下一位置的
接收 API；返回持久接收回执，保留当前 durability witness、限额与崩溃语义。位置和 ingest_time
由可信运行时赋值，客户端不能指定它们的权威值。

## 2. core-store：非法输入与认证失败的持久入口审计

`Store::ingest` 只接收有效 EventEnvelope；`settle_duplicate` 只处理已有合法 D 的重复/冲突。
非法 Protobuf、来源认证失败不能伪装成 observation / maintenance 放入 H，也不能只写 tracing
日志冒充可恢复入口审计。`prepare` 的未发布孤儿对象不是持久日志。

期望提供独立的有界入口拒绝审计接口，保存在同一 core-store 的 D/入口审计域，不改变 H、x、R、θ、L
或时间锚点。持久化消息须来自 IDL；不得记录终端令牌、任何凭据或原始认证帧。失败/满额时必须明确
拒绝，不能先回接收成功。需要配套恢复、容量、非法输入不成长的测试。

## 3. 契约拥有卡：终端认证、分派和对象传输

当前 IDL 有 EventEnvelope（payload/provenance 均为 ObjectRef）、九类 4.2 载荷及 Receipt，但没有
认证请求、认证绑定、RPC 分派封套、载荷/来源对象上传与其回执的权威消息。#57 的当前提交也没有补上
这些接入消息。仅凭 EventEnvelope 无法把客户端尚未保存的 payload/provenance 对象送到核心。

期望由标注契约变更的卡，依 ADR 0007 的只增规则定义并生成这些类型、兼容检查与黄金向量；同时
明确实例/安装身份如何绑定到终端令牌、重连会话、能力与对象预算。不要以手写 JSON、Serde enum、
在旧消息的字符串/bytes 字段内偷塞私有格式，或把 bearer token 当普通日志解决此缺口。

## 4. 可重试拒绝

StoreConfig 已具 D 条数/字节限额；拒绝目前是 `Error::Rejected("durable ingress backpressure")`。
期望公开稳定的有类型背压错误，及终端契约中的可重试拒绝表示。不能依赖英文诊断字符串成为跨进程协议，
也不能把 I/O 不确定结果当作已安全拒绝后换新 ID 重试。

## 不受阻塞的交付

P5-03 已在自己的目录实现有界帧、九类事件验证、DPAPI 令牌和认证能力、当前用户 ACL 的管道组件、
以及只对已由可信运行时正规化事件调用 core-store 的接收适配。完整 daemon 的认证上传分派、拒绝审计、
恢复尾位置和背压回执等待以上接口。不会在此前启用一个声称已安全接通的服务。
