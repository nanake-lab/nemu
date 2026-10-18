# P0-04 doctor 诊断补充

`doctor.txt` 是最终 VRoid 名称匹配修正前的全量输出。之后通过只读 `Get-StartApps` 查询确认开始菜单项为：

- 名称：`VRoidStudio 2.14.0`
- AppID：`D:\VRoidStudio\VRoidStudio.exe`

旧匹配式只接受 `VRoid Studio`，因此产生误报。`tools/doctor.ps1` 现接受 `VRoidStudio 2.14.0` 和 `VRoid Studio` 两种名称，且拒绝无关条目；对应的 3 个纯函数单测通过。由于用户报告异常重启，没有重新运行 Blender、GPU、uv/PyTorch、Low IL 等全量探针，因此 `doctor.txt` 中 VRoid 行仍保留原始值，后续全量复验时以当前脚本为准。
