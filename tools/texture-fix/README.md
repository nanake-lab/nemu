# 贴图脚本处理

只写新 PNG；拒绝覆盖输入、现有文件及 `content/`、`content-template/`。需要 Pillow 12.3.0（MIT-CMU，仅开发工具）。不联网，不替用户选择颜色。

```powershell
python tools/texture-fix/texture_fix.py input.png tools/asset-pipeline/.cache/fixed.png --config operations.json
```

配置示例（数值仅为操作示例，不是 Nemu 配色）：

```json
{"operations":[{"kind":"edgeExtend","iterations":2},{"kind":"palette","colors":[[0,0,0],[255,255,255]]},{"kind":"despeckle","minimumNeighbors":6}]}
```

`edgeExtend` 只扩展全透明像素下的 RGB，保留全部 alpha 和可见像素；`palette` 按 RGB 欧氏距离量化，保留 alpha；`despeckle` 只替换被显式数量的同色邻居包围的孤立色点，保留 alpha。原始图片始终只读。通过 P2-02 选择后的操作参数再用于实际贴图。
