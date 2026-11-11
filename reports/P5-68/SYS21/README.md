# SYS21 契约证据

首审修复追加：当前黄金向量为 565 合法/696 非法。`review-role-red.log` 保存修改语义层之前的 9 路失败；`review-role-green.log` 保存修复后 9 路拒绝与 9 路目标更新接受；`review-contracts.log` 是本轮每语言 14,755 例的完整通过记录。下面的 564/693 和覆盖计数为首版原始证据，不改写。

契约实现：`a85be8a0a6d55cd25ce9aa42b9ce81336b348d74`，main 基线 `a7e8ae8790d297ac697956f6f1256d2e2b9403d5`。本机契约验收时对应代码尚未提交；随后提交未改变受测契约代码。后续只给 Rust 缓存单元测试探针补一行语义边界调用，回归日志见主报告。

- [contracts-02.log](contracts-02.log)：`pnpm test:core-contracts` 完整通过，三语言各 14,718 例（3,257 合法、11,461 非法）。包括官方 protoc、双编码与摘要、SYS17 契约前提和既有 θ 恢复检查。
- [generate-check-01.log](generate-check-01.log)、[compat-01.log](compat-01.log)：生成无过期文件，全部基线 0 违例。
- [contracts-01.log](contracts-01.log)：首轮 Rust/Python 通过，TypeScript 子进程启动失败；[诊断](ts-startup-diagnostic-01.log)显示缓存输出目录找不到 zod。补同版本依赖的目录链接后复跑，未改断言。
- freeze-01 至 freeze-03 保存追加冻结的原始输出。candidate-01 JSON 保存首次尚未发布的新增向量候选；为阻止空槽 Protobuf 被旧消息读取，新增 law_version 必填字段后重新冻结本卡候选。原有 527 合法、623 非法向量始终逐项保留。最终黄金文件为 564 合法、693 非法（本卡新增 37/70），以仓库 golden/ 为准。
- probe-build-* 与 ts-probe-01 是三语言探针编译输出。
- [逐消息覆盖计数](new-message-coverage.json)：九种新消息均满足至少 3 合法、3 非法；从最终冻结 golden 文件统计。

命令环境：Windows；Node 24.21.0、pnpm 12.11.2、Rust 1.99.0；Python 3.12。沿用锁文件依赖与 D 盘缓存，无运行时依赖新增。

```powershell
$env:npm_config_verifyDepsBeforeRun='false'
$env:UV_PROJECT_ENVIRONMENT='D:/Dev/cache/P5-36/contracts-venv'
$env:UV_CACHE_DIR='D:/Dev/cache/uv'
$env:PYTHONPYCACHEPREFIX='D:/Dev/cache/P5-68/pycache'
pnpm --config.verifyDepsBeforeRun=false test:core-contracts
python core/contracts/generate.py --check
python core/contracts/check_compat.py
```

pnpm 的自动依赖校验不支持本机复用的 node_modules junction，故仅本次进程关闭自动安装检查；测试与类型检查均真实执行。锁文件未改。缓存输出目录补有指向同一 node_modules 的链接，以满足 Node 按真实路径解析依赖。
