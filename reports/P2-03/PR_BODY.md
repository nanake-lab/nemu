将 `asset:build` 占位替换为 Blender 5.2.2 / VRM 扩展 4.10.2 的串行无头管线，并新增 `asset:step`。九个独立脚本逐步保存 blend；配置 schema、字段诊断、源检查、尾巴/UV/骨链权重、弹簧/碰撞组、三类表情绑定、MToon、贴图和 v2 profile 均已实现。补充独立贴图 CLI 与 SETUP 的 VRoid 导出设置。

输入 content 和 content-template 保持只读，源文件与模板 21/21 字节数/SHA-256 一致。样例来源、许可、原始失败和逐步成功日志见 `reports/P2-03/REPORT.md`。

验证：
- 两次完整样例构建的 glTF JSON 完全一致，未排除字段；整个 VRM SHA-256 也相同。
- asset-lint 原始 3E/3W：排除示例缺少的左右猫耳链与 Nemu 专属表情后，剩余错误 0。正式构建保留原始 lint 退出码，没有忽略开关。
- 无 Blender pytest 21 项；mypy --strict（18 文件）、ruff、根 lint/typecheck 通过；Vitest 29 文件/443 项通过。Python 测试在本机执行，未改 CI 原四项目循环。
- 版本错误、不同许可 URL、覆盖贴图输入、Runner.Worker 冲突均验证拒绝。

VRM 扩展固定的 licenseUrl 与 CONTRACTS 1.5 存在字段边界：配置为不同 URL 时明确拒绝；没有改契约或静默替换语义。样例使用原模型公共许可。真实 Nemu 参数与外观验收仍在后续卡。

请 Claude 按中难度卡审查。本 PR 不合并，不取下一张卡；最终 CI 状态单独核对。
