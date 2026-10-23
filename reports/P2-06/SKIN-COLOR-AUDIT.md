# P2-06 脸部肤色与打光 — 色彩管理审计及候选说明

## 先报告的问题
旧 P2-06 候选预览没有复用正式舞台的灯光：主光 2.1、环境光 0.65，而舞台为 1.1、0.45。主光方向也不同，不能把旧候选图当作舞台最终肤色。旧预览使用 NoToneMapping 默认值，正式舞台与正式评审台的 AvatarRenderer 则显式使用 LinearToneMapping；是否造成像素差异要用同光照对照确认，不能仅凭配置名称不同就断言为变灰原因。

这次先做同模型、同机位的隔离对照，再单独提供皮肤参数。没有修改正式舞台、全局渲染器、源 VRoid、源 VRM、底色贴图或此前候选文件。

## 核对的路径与规范
锁定版本：three.js 0.174.0、three-vrm 3.5.5。

- `packages/avatar-runtime/src/renderer.ts`：输出 SRGBColorSpace，色调映射 LinearToneMapping，曝光沿用默认 1。舞台和正式评审台均使用该类。默认路径为直接 defaultMSAA，没有 EffectComposer 后处理。
- `tools/asset-pipeline/expressions/preview.mjs`：旧 P2-06 预览，输出 sRGB，但使用独立灯光和默认 NoToneMapping。
- `packages/avatar-runtime/src/loader.ts`：GLTFLoader + VRMLoaderPlugin，无额外手动 gamma 转换。
- three.js GLTFLoader 为 baseColor、emissive PNG 指定 sRGB；three-vrm MToon 插件为 shade、MatCap、rim 的颜色贴图指定 sRGB。normal、outline 宽度等数据贴图使用无色彩空间。颜色因子按 linear-sRGB 数值读取；不能把它们再作为 sRGB 重复解码。
- MToon shader 包含 tonemapping_fragment 和 colorspace_fragment，输出转换不应额外叠加一次。VRM 1.0 同时带有 MToon 与 unlit 时，MToon 优先，插件移除备用 unlit 扩展是预期行为。

官方依据（插件 Context7 额度耗尽后改查锁定版本源码）：
- [three.js r174 色彩管理](https://github.com/mrdoob/three.js/blob/r174/docs/manual/en/introduction/Color-management.html)
- [three.js r174 GLTFLoader](https://github.com/mrdoob/three.js/blob/r174/examples/jsm/loaders/GLTFLoader.js)
- [three.js r174 LinearToneMapping](https://github.com/mrdoob/three.js/blob/r174/src/renderers/shaders/ShaderChunk/tonemapping_pars_fragment.glsl.js)
- [three-vrm 3.5.5 MToon 贴图指定](https://github.com/pixiv/three-vrm/blob/v3.5.5/packages/three-vrm-materials-mtoon/src/GLTFMToonMaterialParamsAssignHelper.ts)
- [three-vrm 3.5.5 MToon shader](https://github.com/pixiv/three-vrm/blob/v3.5.5/packages/three-vrm-materials-mtoon/src/shaders/mtoon.frag)
- [MToon 1.0 规范](https://github.com/vrm-c/vrm-specification/tree/master/specification/VRMC_materials_mtoon-1.0)

## 贴图证据
`skin-audit/texture-evidence.json` 保存纹理索引、尺寸、色值范围和 SHA256，`raw-textures/` 为逐字节提取副本。脸部 emissive 与 MatCap 的全部 RGB 都为 0，不能把它们当作本模型灰白感的额外发光来源。

原底色贴图本身是很浅的桃肤色，包含细微明暗及既有面颊细节。`base-color-samples.json` 只列原贴图固定采样点，不把全图平均（含眼睛、口腔和透明区）冒充肤色。原贴图还含眉、眼和口腔等其他 UV 区块，任何未来调色都不能整张统一染色。

## 独立的皮肤候选
不替换已有 soft / standard / contrast 三组整体风格。以原 standard 模型为固定底板，仅以下两个材质可变：

- `N00_000_00_Face_00_SKIN (Instance)`
- `N00_000_00_Body_00_SKIN (Instance)`

| 候选 | 阴影方向 | 明暗交界 | 参数化轮廓光 |
| --- | --- | --- | --- |
| A | 暖杏 | 柔和，toony=0.35 | 弱 |
| B | 蜜桃 | 更柔和，toony=0.25 | 中 |
| C | 暖金 | 柔和，toony=0.45 | 强 |

完整 linear-sRGB 因子与参数在 `content/_candidates/P2-06/skin/skin-materials.candidates.yaml`，生成 `materials.skin-*.yaml` 管线片段和 `runtime-patches.json`。底色因子、所有贴图字节、其他材质、表情、尾巴、弹簧骨、元数据和使用权限不变；核验见 `candidate-integrity.json`。临时模型只放忽略的 `.cache/P2-06-skin-lint/`，避免复制三份内容模型作为新正式资产。

对照使用正式 AvatarRenderer 的 defaultMSAA、sRGB 输出、LinearToneMapping、曝光 1，以及舞台默认的灯色和强度。仅将主光/背光方向整体左右镜像；所有候选使用相同光照。白、黑、彩色渐变背景，脸部与上半身各一组，包含原 standard 控制列。截图为固定机位、零时间推进的 SwiftShader 预览，不代替真实显示器上的审美判断，不更新视觉基准。

## 如果还希望受光面的底色更暖：两种做法待选
阴影材质只能改变受光分布及阴影色；无法在不动底色的前提下，让所有完全受光区域都获得新的固有肤色。这不是认定源贴图有技术错误。

| 路线 | 做法与结果 | 代价与保全 |
| --- | --- | --- |
| VRoid 新源版本 | 在 VRoid 的肤色参数中定色，另存新的源版本并重新导出；让面部与身体的皮肤在源头保持一致 | 后续重新导出可延续该肤色。需检查所有导出参数并重跑完整管线；原源文件不覆盖。无需手绘 |
| texture-fix 皮肤蒙版调色 | 从皮肤材质 UV 生成蒙版，在蒙版内做连续色彩变换，保留 alpha、纹理渐变及非皮肤区域；面部/身体同步验证接缝 | 可重复、参数化、局部调整，但以后更换 VRoid 贴图需重新匹配蒙版。原工具只有边缘扩展、色板量化、去杂点，没有现成的连续肤色/UV 蒙版操作；选这条后需按目录规则提出并实现该小扩展。不能拿色板量化代替平滑肤色调色 |

两条路线均未执行。参数和贴图输出都应先留在 candidates，经用户选择后才进入正式内容。没有安排手绘或手工修图。

## 当前检查状态
候选参数和数据保全检查已完成。浏览器运行时色彩探针、实际舞台/评审台核验及截图受主机正在执行的 Windows CI 独占窗口约束；完成后把实测结果追加在下节。此处不提前宣布色彩管理已通过。


## 空闲窗口后的实测结果（2026-10-23）
上述窗口等待已解除。实际舞台与正式评审台运行时均为 sRGB 输出、LinearToneMapping、曝光 1；working color space 为 linear-sRGB。脸/身体的颜色贴图标为 sRGB，法线/轮廓宽度等数据贴图不做颜色转换，符合当前锁定版本的 GLTFLoader / three-vrm MToon 规则。三个色卡探针均逐通道精确命中期望值。同一灯光下 NoToneMapping 与 LinearToneMapping（曝光 1）为 **0 个差异像素**。本轮未发现重复 gamma、漏转 sRGB 或色调映射整体冲淡颜色的问题，未修改正式渲染器或生产灯光。

已发现的可比性问题是旧候选预览灯光比舞台更亮：同材质同机位下，旧灯与舞台灯有 219346 个差异像素、最大通道差 74；正式评审台与舞台的灯位/强度也稍有差别。肤色挑选页统一使用舞台灯光强度，左右仅镜像灯向。旧表情/整体材质截图保留，并在入口标注此限制。

原底色固定采样点为 sRGB (250,234,220) 与 (250,233,219)，无光照诊断也显示很浅的桃色；因此目前不能断言底色文件存在技术错误。三组新候选改善阴影暖意；是否仍要让受光面更暖，交由用户选择上面的底色路线。

- [肤色挑选页](skin-audit/review.html)：对照 / A / B / C，3 背景 × 2 光向 × 2 机位，共 48 张、12 张横排对照页。
- `runtime-audit.json`：实际舞台、正式评审台、贴图标记、色卡与 tone mapping 探针；浏览器错误 0。
- `visual/screenshots.json`：48 张双截图一致，附运行时材质值；`visual-verification.json`：36 张候选均在排除标题区后与对照有实际像素差异，运行时阴影因子与候选一致。
- `candidate-integrity.json`：底色因子、贴图/二进制、非皮肤材质及表情/弹簧扩展不变；三份缓存候选分别对应记录的 SHA256。
- `lint-A/B/C.json`：全部 asset-lint 规则均 **0 错误、2 警告**；警告仍为整体材质数量及蒙皮关节的建议值，与既有预览一致，不放宽规则。

截图直接加载已 lint 的缓存候选 GLB，不依赖实时修改材质对象。开发时曾发现实时修改路径画面不变，已撤掉该路径；最初尝试的截图没有作为最终证据。最终像素校验会对相同画面的假候选直接失败。生产 Blender 完整往返及用户选择仍未完成，本报告不将候选验收等同于整卡完成。
