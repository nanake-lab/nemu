# 弹簧候选核验

`verify_candidates.py` 检查完整管线的原始 lint 结果，单独列出 A1.4/A1.5 的错误，保留全部规则的错误数。另逐字段比较源/产物的使用权限、表情 override 和自定义表情名称。

```powershell
D:/Dev/nemu/tools/asset-pipeline/.venv/Scripts/python.exe tools/asset-pipeline/springs/verify_candidates.py --source D:/Dev/nemu/content/nemu/source/nemu-vroid-export.vrm --products tools/asset-pipeline/.cache/candidates-v3 --report reports/P2-05/product-audit-v3.json
node apps/stage/test/jitter/review-candidates.mjs
```

视觉与抖动脚本在既有评审台加载实际生成的 VRM/profile，并核对模型版本和 `joint.settings`，不用旧面板滑块的无效参数。每个模型记录 120 Hz ×30 秒驱动，再记录 5 秒回稳。明确记录位移/转头输入、逐关节角速度、运行 GPU，不把有限值检查冒充高频抖动验收。最终人工手感和系统级拖拽仍待用户与评审台维护方核验。
