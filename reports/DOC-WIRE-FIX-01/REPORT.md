轻档。本次只改文档，订正了说话样例的来源、选法和 P5-38 的开工前提。\
模式 B 按她当下的 m 选样例，P5-38 可以先做念头的保存与接续，3080 实测留到 P5-39。\
请审查这两项修订。草稿 PR 等待用户确认。

## 改动与依据

| 位置 | 依据 | 新读法 |
| --- | --- | --- |
| [ADR 0045 文末追加](../../docs/adr/0045-speak-think-wiring.md#追加2026-11-20d10-回到规格) | [v2.0 §9.7](../../docs/echo-core/echo-core-v2.0-first-principles.md#9-接口)、[ADR 0003 追加](../../docs/adr/0003-speech-modes.md#追加2026-10-19adr-0009-的修订)、[ADR 0009](../../docs/adr/0009-echo-core-v2-first-principles.md#决定) | 模式 A 按 m 生成三语库，用户签字后入库。模式 B 逐次选样。出站 P 中的样例仍单独标注、计量，不能作为自引。 |
| [P6-04「说话样例」](../../tasks/P6-presence.md#p6-04-模式-b云端说话与原文来由投影) | [P5-17 签字](../../tasks/P5-echo-core.md#p5-17-genesis-校准你参与)、[ADR 0003 的 J_text](../../docs/adr/0003-speech-modes.md#模式-b云端投影代理实验) | 从签字库中按固定度量取与当前 m 最近的 k 条同语言样例。 |
| [P6-04「样例换版与重放」](../../tasks/P6-presence.md#p6-04-模式-b云端说话与原文来由投影) | [ADR 0003 的 JT01、JT02](../../docs/adr/0003-speech-modes.md#模式-b云端投影代理实验)、[本卡第 2 节](../../tasks/requests/DOC-WIRE-FIX-01.md#2-p6-04-卡面) | 验相同输入的选样及顺序、零 m 的中性选样，再用绑定的全部选样输入重建实发字节。两版各自重放、缺旧资源时报不可重建的验收继续执行。 |
| [P5-38「开工前提」](../../tasks/P5-echo-core.md#p5-38-持续认知原子发布接续与恢复adr-0026-第-2-步) | [用户决定第 2 项](../../tasks/requests/DOC-WIRE-FIX-01.md#用户决定)、[P5-39 的 COG-A10](../../tasks/P5-echo-core.md#p5-39-持续认知载体调度与行动接入adr-0026-第-3-步) | P0-10 完成即可开工。大脑、声音和渲染同跑的显存与延迟由 P5-39 按原判据实测。 |
| [差异登记 V2-188](../../docs/traceability/diff-register.md#doc-wire-fix-012026-11-20语气样例与-p5-38-开工前提) | [任务原文](../../tasks/requests/DOC-WIRE-FIX-01.md)、[ADR 0045 追加](../../docs/adr/0045-speak-think-wiring.md#追加2026-11-20d10-回到规格) | 接续 V2-186，登记 D10 的两处取代和 COG-A10 的验收归属。 |

## 核验

- `git diff --check origin/main` 通过。29 处新增 Markdown 相对链接指向 13 个不同目标，文件和标题锚点均已核实。
- 与基线对照，ADR 0045 和差异登记均只在末尾追加。P5 卡面只改 P5-38 的一行，P6 卡面只改 P6-04 的两条。
- 开 PR 前及交付前均执行了 `git fetch origin`。交付时 main 为 `469262e4`，与另外 15 个开放 PR 一并核对后，确认 V2-188 无冲突。
- 本卡完成的是文档核验。JT01、JT02 与字节重放由 P6-04 实测，COG-A10 由 P5-39 实测。
