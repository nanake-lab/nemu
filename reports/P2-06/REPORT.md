# P2-06 第二轮表情候选（2026-10-24）

## 当前状态
部分完成。PR #59 保持草稿 / needs-user，等待本轮候选选择及最终 12 组合签字。P2-05 #48 已合并；已合并最新 main 0b5412e 及内容 e62dc1e，原导出器依赖已解除。同步 main 后全量 typecheck / test 的语音模块问题另列 BLOCKED.md，未越界修改。下一张卡：不领取。

## 交付物
- `content/_candidates/P2-06/revision2/`：三套新表情候选、模型、profile 和参数片段；新模型以已合并的 P2-05 成品为底板，弹簧骨保持不变。旧候选和截图保留。
- `revision2/review.html`：第二轮评审，8 个自定义表情及 12 种组合；54 张主截图、7 张诊断图、22 张对照页。白/黑背景包含泪珠、怒符、嘟嘴，整体组合另有三背景。
- `tools/asset-pipeline/expressions/review_server.py`、`review-form.mjs`：只绑定 127.0.0.1，提交选项时在候选目录追加回执；浏览器草稿自动保存、刷新恢复、失败明确提示、可导出。提交绑定候选指纹，不自动定稿。
- `recovered-selection-2026-10-24.json`：刷新前从用户现有 Edge 页面读取并保留选择。沿用 Sparkle A、Blush C、neutral B、happy B、relaxed A、excited B、contrast。worried B 按本次文字反馈继续修改；新组合仍需确认。

## 本轮修改
- Smug：双眼平衡地微眯、单侧嘴角上挑，去掉旧版单眼半闭的组合。
- CatMouth：降低变形幅度。Teary：单侧更大的泪珠，加高光。
- Sleepy：每套均完整闭眼，差别在小嘴/微笑；不会用降低整体强度得到半睁眼。
- Pout：收窄嘴形并向前嘟，重新制作局部形态键。
- AngryMark：圆角、有边框的漫画怒符，并加入眉、眼、嘴。源眉毛变形可向下移动约 28.6 mm；新候选移除整体平移，仅保留受限的相对倾斜，三档竖向位移上限为 5/6.5/8 mm，输出实测最大值为 3/3.5/4.17 mm。emo angry 不再叠加源整体愤怒与额外眉毛。
- worried/sad 的泪珠、sleepy 全闭眼、surprised 降幅、shy/smug/pout 均重组。原设定格只供参考，不修改设定集原文件。
- 嘴下弧线：是下唇下方凹槽的明暗线；关描边/法线贴图仍在，无光照时消失。只在候选中平滑该局部的顶点法线，位置、UV、底图、下巴轮廓均保留。三个新模型所有表情共用此处理，见 `revision2/sheets/mouth.png`。

## 保全
所有原始 VRoid/VRM、底色贴图、P2-05 候选及正式内容均未改。局部法线改动有显式范围与单位长度校验；不移动基准顶点。源二进制前缀、蒙皮、弹簧扩展、使用权限均保持。正式 build.yaml 尚未写入 P2-06 参数。

## 验收
| 检查 | 结果 | 证据 |
| --- | --- | --- |
| 三模型 asset-lint 全部规则 | 各 0 错误、2 警告；仍为材质数 13/建议12、关节277/建议250 | revision2/lint-A/B/C.json |
| 截图确定性 | 54 张主图及诊断图重复渲染一致；浏览器错误0 | revision2/visual/screenshots.json |
| 全闭眼、单侧泪珠、眉毛限位、局部法线及源数据保全 | 通过 | revision2/structural-verification.json |
| 提交 HTTP 边界 | 2 tests 通过：追加保存、非法参数、旧指纹、跨源拒绝 | revision2/server-tests.log |
| 页面交互 | 草稿刷新恢复、提交载荷/回执、失败保留草稿通过；测试 POST 被拦截，没有代替用户提交 | revision2/page-verification.json |
| pipeline Python | 31 tests 通过 | revision2/pytest.log |
| 新 Python strict mypy / ruff，root lint | 通过 | revision2/mypy-final.log、ruff.log、lint.log |
| root typecheck | 失败：上游 sensevoice 返回缺 confidence，及合并后的语音包本地依赖链接缺失 | revision2/typecheck.log |
| root test | 500 tests 通过；7 个语音套件因 zod / contracts 解析失败未运行 | revision2/test-summary.log |

保留了导航等待 load 超时的初次日志；改为等待文档响应后检查实际 ready 标记，未放宽确定性或画面验收。CI/Electron 占用时资源保护会在渲染前停止，相关诊断日志保留。内部形态复核的中间帧位于 D:/Dev/cache/P2-06/preview-pass1，不混入最终截图清单。

## 新增依赖
无。服务使用 Python 标准库；渲染沿用现有 three-vrm/Playwright。

## 待用户确认
- [ ] 本轮自定义表情与 emo 组合；提交本轮选择后，按实际选定的单项重建最终组合，再逐项签字。
- [ ] 本轮去除嘴下明暗线的效果。
- 保持原版皮肤，不执行底色或 VRoid 修改；模型本体外观留给 P2-02/N1 后续评审。
