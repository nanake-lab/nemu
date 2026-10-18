# P2-02 至 P2-06 与 P0-05 依赖状态

检查日期：2026-10-18（用户客户端日期）

| 卡 | 当前状态 | 证据与解锁条件 |
| --- | --- | --- |
| P2-02 VRoid 塑形 | 用户负责；尚未确认完成 | `content/nemu/source/` 仅有原始导出 `nemu-vroid-export.vrm`；没有 `.vroid` 工程、progress 截图或可识别的第一轮中期导出。P2-01 已完成，塑形、外观选择与中期导出由用户完成。原件 SHA-256：`5484C94F54F6C000767FEBACBC99FD7EA23AC074D9CA8EECEBAF65B967DE39AB`。 |
| P2-03 Blender 无头资产管线 | 阻塞，尚未开工 | P2-04 已合并；仍依赖 P0-05 第二部分。当前检测到 Blender 4.5.14，低于项目 `>=5.1` 要求，VRM Add-on 启用状态未通过检查；版本锁文件由 P2-03 建立，存在 `BLOCKED.md` 所述循环。还需 P2-03 开始前阅读 `.agents/skills/blender-headless/SKILL.md`。 |
| P2-04 资产契约校验器 | 已完成并合并 | PR #18 已于 2026-10-18 squash 合并，merge commit `2c5a1928b39a88065d5bcdcd3823bddc8fd693ec`。实现与原始 VRM 诊断见 `reports/P2-04/REPORT.md`。 |
| P2-05 尾巴、猫耳与弹簧骨调校 | 阻塞 | 等 P2-02 中期 VRM、P2-03 和 P1-03。P1-03 已合并；尾巴形状、弹簧手感与候选选择需要用户评审。 |
| P2-06 自定义表情与材质精修 | 阻塞 | 等 P2-02 中期 VRM、P2-03 和 P1-03。P1-03 已合并；表情组合和 MToon 预设需用户逐项评审。 |

## P0-05 剩余依赖

- 第一部分的基础工具链多数已由本机 doctor 检查通过；Codex 与 Antigravity 三线程能力仍未核验。
- 第二部分未完成：当前检出没有 `.gitmodules`，未登记 `nemu-content` 子模块；doctor 尚有缺项；持久 Windows runner 注册也没有完成证据。
- VRoid Studio 2.14.0 被开始菜单独立查询确认；doctor 的名称误报已修复，但全量复验尚未执行。
- 当前没有可直接开工的 Codex-owned P2 下游卡。P2-03 等待 P0-05 第二部分，P2-05/P2-06 还等待 P2-02 中期导出。
