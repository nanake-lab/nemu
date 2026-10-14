# P0-00 借鉴清单（参考项目实测） — 报告

## 结果
完成：参考项目评估调研、代码借鉴清单（AIRI MIT）、交互设计提炼（9 条）、社区高频缺陷防范策略（18 条）、Codex Pet 交互范式剖析以及用户亲自实测评价（AIRI、Mate-Engine、Codex Pet）均已全部完成并闭环。

## 交付物
- `reports/P0-00/borrow-list.md`：详细的结构化借鉴清单，包含可借用代码（AIRI MIT）、交互设计优秀实践（9 条）、必须规避的架构与工程陷阱、社区高频缺陷对照表（18 条高频问题与应对策略）、Codex Desktop Pet 状态机范式剖析，以及用户实测实录与归纳分析。
- `reports/P0-00/user-feedback.txt`：用户亲自撰写的三个参考项目实测评价全文存档。
- `reports/P0-00/REPORT.md`：本报告。
- `reports/P0-00/screenshots/`：相关佐证文件目录。

## 验收
| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| 清单中每条可借用代码都有可访问的仓库路径和许可证 | 通过 | 见 [borrow-list.md](file:///d:/Dev/nemu-antigravity/reports/P0-00/borrow-list.md#L11-L38)，包含 AIRI 仓库相对路径、具体导出函数及 MIT 许可证 |
| 至少 8 条具体交互设计观察并映射到 Nemu 任务卡 | 通过 | 见 [borrow-list.md](file:///d:/Dev/nemu-antigravity/reports/P0-00/borrow-list.md#L40-L75)，提炼 9 条具体观察并对应至 P1-01、P1-02、P3-01、P3-03、P3-04、P4-01、P4-04、P6-01 |
| 透明窗口、VRM 渲染、语音延迟三大领域高频 Issue 各 3 条（AIRI & Mate-Engine 共 18 条） | 通过 | 见 [borrow-list.md](file:///d:/Dev/nemu-antigravity/reports/P0-00/borrow-list.md#L110-L162)，全部附带真实可查证的 GitHub Issue 编号与链接 |
| 本机实际安装并启动验证 | 通过 | 本机验证 `D:\AIRI\airi.exe`，启动成功（PID 60396），日志见 `C:\Users\nanake\AppData\Roaming\ai.moeru.airi\logs\airi-tamagotchi-1791946991170.log` |
| 你在报告末尾补充了试用感受 | 通过 | 见 [borrow-list.md](file:///d:/Dev/nemu-antigravity/reports/P0-00/borrow-list.md#L162-L215) 第 7 节及 [user-feedback.txt](file:///d:/Dev/nemu-antigravity/reports/P0-00/user-feedback.txt)，用户已亲自补充并完成归纳 |

## 新增依赖
| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| 无 | — | — | 本任务卡为调研评估卡，只在 `reports/P0-00/` 下编写报告，未引入任何应用运行时依赖 |

## 待用户确认
- [x] （已完成）用户在 `borrow-list.md` 第 7 节与 `user-feedback.txt` 中补充了亲自试用感受。目前无阻塞项。

## 后续建议（不在本任务范围内）
1. **P0-03 透明舞台尖兵验证**：建议直接采用 AIRI `canvas-alpha.ts` 中的 WebGL 坐标系映射公式与安全半径算法，但必须将其中的同步 `readPixels` 改写为 WebGL2 PBO + fence sync 异步读回，以严格满足待机 CPU ≤ 3%、GPU ≤ 8% 判定红线。
2. **P1-02 桌面窗口管理**：建议参考 AIRI 的特权协议隔离方案（`protectPrivilegedWindowNavigation`）以及多屏 DPI 下的工作区碰撞边界检测。
3. **P3-01 / P3-03 身体动作**：将用户在 Mate-Engine 体验中高度肯定的精细模型呈现、直接桌面融入（直接拖动、窗口坐立、自主走动）与 Codex Pet 的低打扰分层交互（安静待机、悬停反馈与快捷任务入口）作为身体设计核心，同时避免 Mate-Engine 自主乱走、无反馈换装等缺陷。

## 下一步
本卡验收条件全部满足，准备提交 PR 并合并 `task/P0-00-borrow-list`。  
根据 `tasks/README.md` 第 2 节取卡规则，Antigravity 线程下一张卡为：**P0-03 透明舞台技术验证**。  
因 P0-03 依赖 P0-01，当前 Codex 线程仍在 `task/P0-01-monorepo` 执行中，待 P0-01 合并后 Antigravity 立即开工 P0-03。

