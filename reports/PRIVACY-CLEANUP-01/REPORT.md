# PRIVACY-CLEANUP-01 — 指定高风险项清理

## 结果

已完成用户指定的四项清理。PR #165 标记 needs-user；请审查，不合并、不继续其他清理。依据 AGENTS §6（2026-11-10）及用户 2026-11-11 对两份 DOCX 的一次性授权。

## 逐项交付

| 项目 | 改动与原件位置 |
| --- | --- |
| P0-07 进程快照 | 原 69 条记录按 #149 名称白名单保留 36 条 node.exe，移除 33 条；只提供记录总数、保留项和脱敏说明。原件：私有仓 P0-07/recovery-20261020/preflight-processes.json。原件没有资源指标，resource_totals 为 null，明确未记录；69 是该快照记录数，不宣称整机进程数。原件无路径/命令行，白名单归属未能独立证实。 |
| P5-03 SID 文件 | second-account/user-message.txt 的 SID 改为 [WINDOWS_ACCOUNT_SID]；原件：私有仓 P5-03/PRIVACY-CLEANUP-01/second-account/user-message.txt。 |
| PR #65 评论 6398042388 | GitHub API 已编辑，2 处 SID 改为同一占位符；回读确认其余正文逐字符不变。编辑前原文和完整 API 响应先保存、推送并核验私有仓 P5-03/PRIVACY-CLEANUP-01/comment-before.txt、comment-before.json。 |
| 11 个局域网文件 | 严格取 #163 私有 FINDINGS.csv 的 main / 局域网地址类别：P4-01 的 7 个、P5-04 的 4 个 JSON，各 1 处地址替换为 [LAN_ADDRESS]，其余字节不变。原件：私有仓 PRIVACY-CLEANUP-01/originals/reports/ 下的对应路径；逐文件见索引。 |
| 两份 DOCX | Ada Echo Core Math define.docx、Echo Core Implementation Plan.docx：清空 core.xml 的 creator、lastModifiedBy；移除 custom.xml 的 KSOTemplateDocerSaveRecord（含个人账户标识）。app.xml 无个人/公司名称，保持原样；其他自定义属性未改。原件：私有仓 PRIVACY-CLEANUP-01/originals/docs/echo-core/original/ 下。 |
| 校验同步 | baseline.md 追加新旧哈希对照，旧表保留；verify-baseline.ps1 精确要求两份新哈希，拒绝旧文件冒充当前基线；Docs guard 仅允许两个指定文件的精确旧→新 SHA-256 转换，拒绝任意其他原文变更及回退。AGENTS 的原文保护规则下追加用户指定的一次性例外。 |

所有 17 份原件（15 个文件、评论正文和 API 响应）已在私有证据提交 `9f8f9058fe9cc79743b3073d90391ca3fa9d90ab` 中逐项核对实际 Git blob SHA-256；详见 [EVIDENCE-INDEX](EVIDENCE-INDEX.md)。P0-07、P4-01、P5-03、P5-04 的索引亦已追加入口，既有证据不覆盖。

## DOCX 正文不变证明

[DOCX-MEMBER-SHA256.csv](DOCX-MEMBER-SHA256.csv) 列出两份文档全部 38 个 ZIP 成员的改前、改后 SHA-256。仅 4 个成员变化（每份 core.xml、custom.xml 各一个），其余 34 个成员解压后的原始字节完全一致，包括 word/document.xml、样式、关系、图片及 app.xml；成员集合、顺序和 ZIP 注释不变。未使用 Word 重存或正文重排。此检查是成员字节比较，不是重新渲染或重新验收规格。

## 验证

| 检查 | 结果 |
| --- | --- |
| 17 份私有证据 Git blob 摘要与远端提交 | 通过 |
| 38 个 DOCX 成员哈希、非目标成员字节、属性 XML | 通过 |
| 12 个改动 JSON 完整解析 | 通过 |
| SID 评论回读、逐字符限定差异 | 通过 |
| PowerShell AST 解析及基线脚本的 5 文档哈希/XML 部分 | 通过；未运行无关 Rust 探针及全工作区测试 |
| Docs guard 六个边界用例 | 精确授权转换、无变化接受；新文件篡改、错误旧哈希、逆向转换、其他原文拒绝 |
| YAML 完整解析与原有任务目录检查 | 通过；149 cards，0 problems |
| git diff --check、新增 Markdown 相对链接 | 通过 |

开发检查记录：首次通用 XML 探查误把 docProps 缩略图作为 XML，已限定 .xml；默认 Python 无 YAML 模块，使用已有 P0-10 环境解析，未新增依赖。一次临时基线行匹配检查不兼容换行，已撤回该无关改动；编码检查发现并恢复原脚本中文行，最终只增加新哈希校验。首次本地未发布提交被 GH007 邮箱隐私保护拒绝，改用命令级 GitHub noreply 身份后成功，未修改全局设置或已发布历史。

## 范围与下一步

未改个人路径、主机名、用户名、Actions 日志或产物、CI 日志邮箱、其他 original 文件；Git 历史不改写。历史验证目录的旧哈希是当时结果，保留并由新基线追加说明关联。无运行时依赖、无核心或契约语义变化。

验证基线为 `a6a0567f02efa5a183fa9fdb449c064afe11e661`：开工后 main 合入 #157，已 merge；其 Windows 隐私脚本变化与本次文件脱敏不冲突。本 PR 包含 Docs guard 配置改动，按中档做受影响脚本与工作流检查，不运行产品全套测试。CI 状态以 PR 当前提交为准。

- [ ] （阻塞）请用户审查本次脱敏、两份 DOCX 一次性元数据清理及校验同步。

请审查。停在 PR #165，等待用户决定。
