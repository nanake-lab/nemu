# P5-36：2026-10-30 main 增量核对

## 基线与合并

- 上轮验证的 main 基线 B：`f24f1256259c998bc213a9dc93103fcfb9cfe89a`。
- 首次 fetch 得到的 main M0：`02948dc11aa1b43dc0dfe67c9da2e61676427385`；已验证 B 是 M0 的祖先。
- 验证期间新增 #122/#123，收尾 main M：`fb14feff3c35cf64ae661b642c3455ed506394e2`。仅文档/任务/规则变化，具体影响见 [late-main-impact.json](late-main-impact.json)；本轮完整测试实际基线仍为 M0，源码相同的后继合并复用这些结果。
- 合并前任务 head：`ce255cca0062f38fe51b94283eb9647867a18040`。
- merge 提交：`9a3858ba1d31506cf2a7b2d34dd46f913c9d5594`，两个父提交分别为原任务 head 和 M。
- 验证源码提交：`06ec1885320ab52a3da7bba0276e847d432e2793`，在 merge 后只适配 gateway 的一个直接受影响测试夹具。
- 后续只更新本卡报告/证据；未 rebase、未强推、未自行合并 PR。

首次增量清单见 [main-commits.txt](main-commits.txt)、[main-files.txt](main-files.txt)。B..M0 有实质源码变化，未按“仅文档”免测处理。AGENTS、ARCHITECTURE、CONTRACTS、echo-core README、技能文件及 P5-36 卡本身在首次增量内未变；相关规格和 ADR 增量已读取。M0..M 的必读文档、AGENTS 风险分级与 P5-36 新增读法已核对：本 PR 仍属重档，后继成长设计仍未交付，当前公式/存储与验收命令不变。

## 唯一冲突

`docs/traceability/diff-register.md` 按日期保留 2026-10-28 P5-36 V2-107–109、2026-10-29 P5-23 恢复状态、2026-10-29 ADR 0032 V2-118–136。以 Git 原始 blob 重建，插入本分支原始追加字节后，删除该插入段即可逐字节还原 M 的整个文件；新增段也逐字节等于原任务 head。内容行修改数为 0。

哈希与断言见 [conflict-resolution.json](conflict-resolution.json)。初次脚本的错误假设及未发布中间提交纠正记录见 [resolution-notes.txt](resolution-notes.txt)；最终 merge 无冲突标记。

## 增量影响与检查选择

| main 增量 | 对本卡的影响 | 本轮处理与验证 |
| --- | --- | --- |
| #49：`core-kernel/law`，模块登记，q/V/容量/规律测试 | 工作区新增 G 算子、与既有类型和存储共同构建 | 完整 Rust 工作区测试、fmt、Clippy `-D warnings`；未改 C/G 算法 |
| #64：gateway，Cargo 依赖，Windows W18 | 新增生产上传/存储消费者与临时运行时夹具；旧 θ 夹具触发 superseded 写入门禁 | 原测试先复现拒绝，再最小适配 V2/当前头读取/规律资源版本；完整工作区包含 gateway Windows 测试 |
| #117：`core-kernel/evidence` | 新模块仍未在生产 `lib.rs` 登记，工作区测试无法覆盖它 | 复用上游独立 harness 的构造与命令，在 P5-36 缓存建立同树引用；另跑 test、Clippy、rustfmt；不修改或登记该模块 |
| #118：ADR 0032、v2.0 附录、差异登记 | 新的出生起点/生活基线及证据语义决定影响后继设计；没有发布本卡可自行补写的 C 替代公式 | 保留 main 原文；本轮只验证现有版本状态的完整保存/恢复，不宣称完成 ADR 0032 后继成长语义 |
| #115：ADR 0031、载体方案与任务依赖 | 载体/注入通路后继工作；P5-36 验收及存储代码未变 | 阅读增量，不扩展到载体、迁移、历史重放或运行时接线 |
| #112/#113/#114：研究报告与合成证据 | 无生产源码变化 | 保留原始证据，不运行研究实验 |
| `windows.yml` 新增 gateway 路径与 W18 | PR 转可审查后需要现有 Windows 路径门禁；本卡 W02 仍必要 | 按用户指示保持草稿，确认 runner 在线后由用户转可审查；W02 通过后交增量复审，本轮不等待或查询 CI |
| 本卡 θ 契约/算法在 main 增量内未变，但与上述源码组合变化 | 不能仅沿用旧 head 的完整验收结论 | 重跑契约生成/兼容性/三语言验证、真实跨语言恢复续算及 65 前缀存储提交→关闭→重开→读取→恢复→全后缀续算 |
| 后续 #122/#123：DOC-PRESETS-01 文档路由及 AGENTS 风险分级 | 代码、依赖、生成器、测试、工作流未变；P5-36 卡追加说明保留当前公式和存储 | merge 后核对源码相同；复用刚完成的重档检查，报告首行注明档位，PR 开头按三行白话交接 |

## gateway 最小适配

[开放 PR 实际路径核对](open-pr-gateway-scope.json) 未发现其他开放 PR 占用 `core/crates/gateway/`；P5-08 卡目录是 `core/crates/core-runtime/`，P5-26 的实际修改为 `core-kernel/recompute/`。按用户此前批准的“直接受影响调用点”范围，仅改 `gateway/src/tests/runtime_fixture.rs` 三处：使用合法生成的 ThetaSnapshotV2 夹具、LawVersion 对象版本绑定 `theta.parameters.law_id`、当前头按 V2 读取。

首次既有 SYS01 测试因旧 θ 写入被拒绝（exit 101）保留在 [gateway-integration-initial.log](gateway-integration-initial.log)；修复后同一测试通过，见 [gateway-integration-fixed.json](gateway-integration-fixed.json)。未削弱断言或门禁。生产 `gateway/build.rs` 根据 schema 自动排除 superseded 消息，V2 经 `Store::prepare` 的完整语义验证路径写入，不需要生产补丁。

## 验证边界

所有本轮执行的命令、起止时间、退出码与源码提交记录在同目录各 `*.json`，输出在对应 `*.log`。独立 evidence harness 是上游条件性模块验证，不等于真实 H/L 生产接线；65 前缀恢复/续算也不等于独立完整 H 重算、旧历史迁移/回退或 SYS17。既有 ignored 数值诊断不作通过项。

早期命令记录的 `main` 字段是命令结束时所见的远端引用，远端在验证期间推进，因此该字段不直接作为实际验证基线。`results.json` 的每项 `actual_main_baseline` 按固定测试 head 与最终 main 的祖先关系核定；本轮实际均为 M0。

契约启动脚本首次将单个参数拆开，实际运行根 Vitest；虽然进程 exit 0，但不计契约通过。该输出保留在 `contracts.log`，原因与修正见 [contracts-invocation-incident.json](contracts-invocation-incident.json)，有效重跑输出是 `contracts-corrected.log`。未改仓库实现或验收脚本。

测试使用临时存储及 P5-36 的 D 盘缓存，重负载串行，未访问用户排练或正式存储。原失败证据保留；没有变更超时、验收阈值或系统设置。
