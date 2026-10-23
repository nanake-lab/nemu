# P2-05 候选管线

`prepare_candidates.py` 只把明确的设计参数文件展开为 3×3 份候选 YAML；不自动选择外形或手感。输入为只读 VRoid VRM、bible 和 `content/_candidates/P2-05/design-proposals.json`。配置文件、manifest 和来源哈希仍在候选目录。

```powershell
$env:NEMU_ASSET_PYTHON='D:/Dev/nemu/tools/asset-pipeline/.venv/Scripts/python.exe'
& $env:NEMU_ASSET_PYTHON tools/asset-pipeline/tail/prepare_candidates.py --source D:/Dev/nemu/content/nemu/source/nemu-vroid-export.vrm --bible D:/Dev/nemu/content/nemu/bible --designs D:/Dev/nemu/content/_candidates/P2-05/design-proposals.json --output D:/Dev/nemu/content/_candidates/P2-05
& $env:NEMU_ASSET_PYTHON tools/asset-pipeline/tail/run_candidate.py --config D:/Dev/nemu/content/_candidates/P2-05/build-standard-standard.yaml --output tools/asset-pipeline/.cache/review-candidate
```

候选入口复用现有 `pipeline.main()` 的导入、源校验、弹簧、表情、材质、贴图、导出、profile、manifest 和完整 asset-lint，只把尾巴阶段指向同目录的 `build_candidate.py`。因此 A1.3 仍返回非零状态，没有添加忽略校验的开关。

尾巴阶段复用 `build_tail.execute()` 的几何/权重/UV。新生成的尾巴不能使用源头发合并图集：它的 UV 不在原图集布局中。仅新尾巴材质去掉原图集，关闭原图集依赖的白色发光，并以配置中的色板颜色作为 base/shade；保留其余 MToon 响应。源模型的材质不改。当前候选为纯色体块，供选长度、宽度、曲线，尚未增加毛束细节。

所有 Blender 仍只用无头脚本。运行前需额外确认 P4 测量/安静窗口已结束；入口的进程检查只是第二层检查，不能发现尚未启动测量进程的预约窗口。不要并行运行候选构建、视觉脚本或其他重负载。

原 `pnpm asset:build` 不自动使用这个候选阶段。正式通道挂接需要扩展 `build_tail.py` 的目录范围，见 `tasks/requests/P2-05-tail-stage-integration.md`；本次保留原实现，避免卡外修改。
