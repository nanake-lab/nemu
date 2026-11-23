# P5-07 后继请求：正式命名管道与候选记录合同

2026-11-01 用户明确授权 #96 只交付可独立审查的 worker／bridge 基础层。
P5-07 整卡仍为部分交付，本请求进入契约线排队，不创建临时 IPC。

## 缺口与归属

本卡允许目录仅为 workers/irl/、executors/codex-bridge/src/base/。
正式 IPC 需要 core/contracts/ 的版本化 IDL、生成器、黄金向量以及调用方接线，
超出当前目录。请契约线安排独立契约卡、ADR 和目录窗口，随后由 P5-07 消费。

- 核心→worker、worker→bridge、本地 model-adapter 端点：认证角色、ACL、消息大小、
  对象获取与完整性绑定、失败/取消、生产者与资源版本；不得裸 JSON 或自造消息替代。
- 原 wire schema 的 sourceRef/sourceSpan、predicateLabel、roles/value/modality/time
  到正式语义对象的映射，结合 P5-06 G01–G05。旧 schema 的 p0-08-NNNN 是探针格式；
  本次库内 alias 只用于模型交互，不能当正式 event ID 或绕过 Concept Registry。
- 实际输出及解析候选可恢复保存；SemanticCandidateV2、staged/accepted z、原子发布
  明确区分。c_core 与注册表引用须有受限、可验证的对象载荷，不能只传无法读取的引用。
- ADR 0031 本地、异步、不挡回复：P5-08/19 负责前序顺序、迟到结果、stale 拒绝与重算，
  不在本卡猜测新的提交政策或截止阈值。P5-13 负责真实 m=0/受约束解码与端点。
- helpers 配置由所属卡提供；ADR 0033 的通知/H/恢复与防失控合同归后继，
  不恢复每日 30 次限制、不自定新阈值或费用上限。

## 回归与状态

完成后在 **P5-07 名下**补齐真实命名管道、双端调用、来源注入 SYS02 联测、真实协议
录制回放、失败/断线不盲重发，并对 #96 基础层作增量复审。
当前进程内合成测试不是正式 IPC、OS 隔离或完整 SYS02 通过。

看板当前由开放 PR 推导为待审查，不应标完成。现行 tools/status/build_status.py
会把已合并的同卡 PR 推导为 done：**若先合并基础层，须由看板/任务目录所属卡提供
部分交付状态表达或保持同卡后继开放认领，再核对看板**。本次不改 cards.json 或看板
代码、不合并 #96；最终完成必须等本请求及测量请求补齐并增量复审。
