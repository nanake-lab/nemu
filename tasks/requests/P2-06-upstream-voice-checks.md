# P2-06 请求：main 同步后的语音检查

P2-06 在合并 origin/main 0b5412e 后，root typecheck 报 `packages/voice/src/asr/sensevoice.ts:70`：返回 `{text,lang}` 缺少 AsrResult 要求的 `confidence`。该文件和 packages/contracts/src/appbus.ts 相对 origin/main 无差异。

此外，本任务原有依赖环境没有新合并 packages/voice 的 zod / @nemu/contracts、桌面 @nemu/voice 工作区链接；root test 有500 tests通过、7个语音套件导入失败。本卡未联网安装或修改共享依赖树，也未改语音实现。两类问题应分别处理，不能把依赖环境错误都归为代码缺陷。

期望：语音/终端契约的所有者核对 AsrResult confidence 的兼容实现；本地环境按已合并锁文件恢复依赖链接。修复后 P2-06 会重新同步并执行全量检查。

证据：reports/P2-06/revision2/typecheck.log、test-summary.log。P2-06 自身新增模型全规则 lint 0错误、Python/提交功能验证通过；PR保持草稿，不越界修复。
