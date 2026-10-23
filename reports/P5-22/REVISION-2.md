# P5-22 A1 修订报告

## 结果与位置

已修复复审针对 0dc5e87 的唯一新增代码阻塞 A1；本机验收完成，整体交付部分完成（见 BLOCKED.md：Windows runner 离线，用户暂时无法启动）。审查意见：https://github.com/nanake-lab/nemu/pull/57#issuecomment-6342345330 。独立会话审查及用户批准仍待完成；PR 保持草稿及 needs-user，执行者不合并。

工作区已从 D:\Dev\nemu-p522 迁到用户指定的 D:\Dev\nemu-wt\P5-22，保留同一任务分支。已合并最新 origin/main=876cff7211389f7058668dfb2b9592634d517abc（原先落后的 #63、#58 两个提交），合并提交 870ead6。本轮检查在合并后执行。历史证据仍保留当时的原始路径，未改写。

本轮运行先 dot-source `reports/P5-22/d-drive-env.ps1`：临时文件、uv、Python bytecode、ruff/mypy、Node、npm、pnpm、Cargo、.NET/NuGet 等新缓存使用 D 盘；工作区内 target、node_modules、.venv 均在 D 盘。已有 C 盘工具链仅作为已安装程序读取，没有新建 C 盘工作区。该脚本未修改系统级环境变量。

## 修改与原因

- 模板 `core/contracts/templates/typescript/browser.ts` 不再用会过滤 `__proto__` 的 z.record。字典必须是普通或 null-prototype 对象，枚举 JSON 自有条目，每个键和值分别按权威 IDL descriptor 校验；任何错误均沿原键路径显式报错，解析失败时 resourceToWire 不产生载荷。
- 用 Object.fromEntries 重建数据属性，保留 `__proto__`、`constructor`、`prototype` 等合法字符串键，同时避免触发原型 setter；没有增加特殊名称黑名单。
- 重新生成 `packages/contracts/src/generated/echo/browser.ts`。IDL、资源配置值、数值范围及编码规则均未修改。
- 新测试 `resource-dictionary.test.ts` 检查解析前后键集合、每个自有属性和值、原型不变、原输入/解析结果两条路径的完整 wire 载荷一致；JSON.parse 与 null-prototype 字典均覆盖。非法 -1、null、"invalid"、Infinity、NaN 明确拒绝并保留 cooldownS.__proto__ 错误路径；数组/空键/坏 Unicode 也拒绝。

## 配套修改

| 文件 | 必要性 |
| --- | --- |
| browser.ts 模板及生成副本 | 修复公共资源入口丢键/绕过值校验的根因，保持单一生成源 |
| fixture_inputs.py、golden/legal.json | 只追加两个合法特殊键载荷，纳入三语言日常规范字节/摘要验收；旧向量不重写 |
| resource-dictionary.test.ts | 用冻结的 Rust 权威字节和摘要检查真正的资源投影，避免只测 wire 结构而漏掉资源字典 |
| d-drive-env.ps1 | 记录 D 盘缓存与临时目录配置，便于复现，不修改全局工具设置 |
| run-checks.ps1 的 Only 参数、verify-resource-vectors.py | 原样复跑迁移后受影响的检查、保留两轮证据；单列两个 A1 向量的三语言规范字节/摘要核对 |

无新增依赖，无新增消息/标签、差异登记或编码规则；不处理已披露的非阻塞 C1(a)。所有新增实现模块小于 400 行；fixture_inputs.py 是已有集中夹具模块，本轮只追加两组数据。

## 规格对照

| 条款 | 实现 | 测试 |
| --- | --- | --- |
| CONTRACTS §8；terminal.proto ProactivityCooldown.kind 非空字符串、seconds 有限且非负 | dictionary 按 descriptor 校验每个自有条目，无额外名称限制 | resource-dictionary.test.ts 合法特殊键与非法值矩阵 |
| ADR 0007 §5–8：字符串无隐式归一化、集合排序、规范摘要 | IDL/三语言 codec 不改；资源投影保留原 kind/seconds | 两个新增黄金载荷经 Rust/Python/TypeScript 的 encode/canonical/protobuf/digest 比较 |
| ADR 0007 旧编码保留 | 只追加向量；兼容检查原样执行 | revision-2-frozen-byte-comparison.json；compat.log |

## 验收与字节证据

最终命令、时间、工作目录、退出码及原始日志在 `revision-2-checks/results.json` 和同目录。复现前先 `. ./reports/P5-22/d-drive-env.ps1`，再运行 `./reports/P5-22/run-checks.ps1 -Evidence reports/P5-22/<新目录>`，不要覆盖现有证据。

完整运行的 Python 7 项初次因迁移前的 .venv 启动器绝对路径失效而失败，原始日志保留错误 `uv trampoline failed to canonicalize script path`。旧环境移到 D:\Dev\cache\P5-22\pre-move-venvs 留存，在新位置重建后，受影响的整组 14 项 Python 检查全部通过，见 revision-2-python-checks/results.json 和 revision-2-contract-env-rebuild.log。未改测试或阈值。其余 12 项完整检查原已通过，不重复无关的 Rust 崩溃矩阵。

`revision-2-final-checks.json` 按名称汇总两轮的最新结果：恰好 26 项，全部 exit_code=0，并保留原始日志位置。

| 检查 | 最终结果 | 证据 |
| --- | --- | --- |
| pnpm lint/typecheck/test/build | 全通过；33 文件、481 测试 | revision-2-checks/{lint,typecheck,unit,build}.log |
| pnpm test:contracts | 7 文件、350 测试通过 | revision-2-checks/contracts.log |
| pnpm test:core-contracts / D10.8 / SYS21 | Rust/Python/TypeScript 各 10,766 例通过，2,527 合法、8,239 拒绝；protoc 445 通过 | revision-2-checks/core-contracts.log |
| generate.py --check / check_compat.py --ref origin/main | 0 stale；各基线/current/origin/main 0 违例；8 个 schema 测试通过 | revision-2-checks/{generate,compat,parser}.log |
| Rust fmt/clippy -D warnings/workspace tests | 全通过，包含新 main 的 core-store 崩溃恢复矩阵 | revision-2-checks/rust-*.log |
| 契约工具 ruff/mypy strict、三个 worker sync/ruff/mypy strict/pytest | 重建环境后 14 项全通过 | revision-2-python-checks/ |

定向 contracts：7 文件、350 测试通过（revision-2-targeted.log）。全仓：33 文件、481 测试通过。契约覆盖率语句/行/函数 100%，分支 97.46%；生成目录不在该覆盖率统计中，生成字典实现由上述定向测试及三语言黄金验收覆盖。

`freeze_vectors.py` 先重新验证全部 443 合法、457 非法冻结向量，再只追加 2 合法、0 非法，当前为 445 合法、457 非法、18 Unicode（revision-2-freeze.log）。新增一个 `__proto__:17` 和一个五键混合载荷；前者摘要 b46fb3719dc49a58bde3f243c19f116ab294bf5a495f9d76fe5e446c5cec431d 与审查反例的正确载荷一致。新增向量的完整规范字节、Protobuf 字节及摘要均在 golden/legal.json；资源投影测试直接比较它们，不仅比较对象外形。

旧字节复现：`python reports/P5-22/verify-frozen.py reports/P5-22/<新文件>.json 0dc5e87`。新结果写入 revision-2-frozen-byte-comparison.json；旧首轮和修订一结果保留。

相对被审查 0dc5e87 的原 443 合法、457 非法、18 Unicode 前缀逐项完全一致。三语言各 1,816 查询（1,347 成功、469 拒绝），value、canonical、protobuf、digest 逐项 0 差异。三类旧列表 SHA256：

| 列表 | 基准与当前旧前缀共同 SHA256 |
| --- | --- |
| legal | 2187bdbd7efcd31122be1bff5e9241929be01f2296dadc219044c27057f761e1 |
| illegal | b62912905aabb40ddc18056ff5a22fa4e0acfbf257310e53379d282e6cdc5cb3 |
| unicode | 0e3e6d848402361540fef0b798337c970a598a8c6e78fe7d2ca52a7e5a900f31 |

成功输出列表的哈希在 Rust/Python/TypeScript 与基准中全部一致：Canonical 为 66fbf9be58ac58c464c90c76aa6704ad26efb8ae84fd41fc06accc78b31f178c；Protobuf 为 311311a4f2d4962d03d2beb21265f68fe6a67c44a47b33f201cd5306e3015aee；digest 为 1239ecc3675b0b3bb95b9544023938410b1fae3cc6ce8380ea8e657052ccd86b。

针对两个新增 A1 向量又独立调用三语言探针，完整 value/规范字节/Protobuf/摘要与冻结预期逐项相同，0 差异；见 revision-2-resource-vectors.json / .log。复现：`python reports/P5-22/verify-resource-vectors.py reports/P5-22/<新文件>.json`。该证据与 TypeScript 资源→wire→冻结字节测试相连，证明实际资源投影保留这些条目，未只验证直接 wire 输入。

CI 的最终 head、结果和链接以 PR 回复记录为准；全部验收完成前保持草稿。

修复代码提交 fc5f6dab4d53dd673b0906844ecb66e5d231eb0c；Windows run 39910022507 因 runner 离线排队，用户明确暂时无法启动，未记为通过。本轮结尾的文档提交只记录这一阻塞，不修改该受测代码。

## 限制与下一步

无 GPU、用户正式或排练存储访问；本卡不签发后续成长、主动调度或社交执行运行时验收。合并 main 带入的 core-store 测试只使用其合成临时存储。

完成本轮验收后在 PR 回复「请审查」并停下，由用户交给另开的 Codex 会话；不操作浏览器或其他应用请求审查，不联系其他执行者，不自行合并。
