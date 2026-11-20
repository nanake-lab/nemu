# 执行准备

复用C3 2a的probe及native header；probe唯一逻辑变化为free上限96→384。generate.py准备22个原文prompt、44项固定排序任务及2个工程guard，绑定模型/引擎/向量/源码哈希。运行时监测资源，15分钟硬限（预留30秒退出）；cap或其它故障立即停止，无自动重试。

本地MSVC /W4 /WX编译通过，44项及两个guard清单、原文编号连续性、原probe单点变化、模型/全部引擎文件/方向哈希核验通过。真实生成尚未运行。本附件直接本地验证，沿用#186最简流程，不新增专用CI或依赖。

命令（仅有真实用户授权且runner停止时运行）：python tools/r-steer-pair-02/generate.py run --user-approved

GPU窗口已获用户批准并确认runner停好；只读预检未发现runner。生成前重新执行资源检查。执行证据位置与哈希见EXECUTION-INDEX.json。
