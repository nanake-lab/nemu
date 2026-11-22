# Windows 载体库

在 x64 MSVC Developer PowerShell 中运行下列命令。引擎目录使用 P0-12 校验过的 b11669 Windows CUDA DLL，源码树使用同一提交 `620d2225cc436ab0b001e3200e590e3b286ef9ab`，包含 common、ggml/include、include 和 tools/mtmd。

```powershell
python workers/model-adapter/native/build.py --source $llamaSource --engine $engineDir --output $buildDir
```

将构建的 carrier.dll 与其受信发布摘要交给 `nemu_model_adapter.Assets`。Python 包的 manifest 固定 Q4_K_M、官方 tokenizer、F16 mmproj 和引擎文件的哈希。默认加载 mmproj。A/C 走 `LocalCarrier.generate`，其调用方负责把已授权的输入、事实视图和 C 草稿渲染为本轮材料；IRL、A、M 使用 `LocalCarrier.judgment` 的 m=0 异步受约束流。关闭模型前等待请求释放。只在显式调用构造函数时加载 GPU。

正式信封来自 P5-19 已提交的发布。受信装载方独立确认 genesis 与当前资源之间的迁移、P5-31 的证据资格以及当前 hook，随后传入 `Publication`。本地认知用 `local_result` 绑定同一次运行的探针及输出，实际存储由调用方写入。

`--measurement-control plain` 编译无探针对照，`--measurement-control drop-vector` 编译故意漏注入的 AT13 对照。二者只供测量脚本使用。
