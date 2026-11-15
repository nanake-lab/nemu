# C0 零注入测量接口预注册 v1

状态：运行前冻结；本文件定义测量，不是测量结果或正式资格。用户安静窗口尚未收到。执行前本文件、源码与全部实际提示必须提交推送并核对远端 head。

## 假设、样本与范围

假设 C0：当前载体能可靠使用中性输出槽，四个概念的表达比较有可测空间，代码或位置偏差没有压倒该空间。失败意味着接口不足，不意味着方向失败。

[fixtures.py](fixtures.py) 是完整原文：8个场景根（每概念2个），每根中英日配对翻译；每语言版本2种共同附语×2种正端点代码映射×2种端点显示顺序×2种代码字母表AB/XY，即16布局。共 **384表达选择**；每语言版本另有1次自由greedy回复，共 **24次**。四个客观规则根（加法、明示计数、阈值、最大值），三语×2映射×2顺序，共 **48次oracle选择**。首尾各三语同提示重放，共 **6次完整首位置logits对照**。合计 **462请求**，最多 **1974输出位置**（438×1+24×64）。每个原始输入、token ID、greedy输出、候选logits、logZ、未约束argmax和top12留存；六个控制保留整个词表logits。

八根只是可行性材料，不是八个独立领域，更不是384份独立证据。翻译、代码和布局是重复测量，不能扩大n。附语是两端点同时增加的同一句礼貌话，不改变概念、事实或标签真值。候选回复是离线研究材料，没有要求模型采用性格或人设；无生产p或真实数据。

端点是按v2.0 §3编写的**研究者预标注表达**，未获独立语义验证，可能混入一般正负情感、词频、长度和含义重叠；C0通过不证明构念有效。“好奇正端点”涉及理解的感受，不能凭选择它声称有学习进展。主要行为是实际选择表达，不测工具或服从，不给她增加行为策略。

## 固定执行配置

模型修订与Q4_K_M哈希沿 [P0-12](../P0-12/REPORT.md)。固定llama.cpp b11669 / `620d2225cc436ab0b001e3200e590e3b286ef9ab`，55文件与P0-12清单核对。只读复用native_probe.hpp的DLL加载、CUDA拒绝CPU回落、shape与clear；32层/2560维/248320词表。context/batch/ubatch512、threads4、Flash Attention off，greedy；指标temperature1。没有随机采样种子，不用确定性重放估计种子方差。无视觉编码器或图像，不解释图文行为。

每请求先`llama_memory_clear(..., true)`同时清attention/recurrent，再撤除control vector、完整prefill。第15层只是Native构造参数，C0不注入、不观测层、不作选层。tokenize沿既有460上限；64生成上限与context512匹配；超长、shape错误和任意非有限数即停，不裁剪。非思考模板与P0-12冻结的英文实际提示逐字核对，输入空think前缀不计作输出思考。

槽只观测**真实未约束首token argmax**，不通过mask强行变成合法答案。另报`q=exp(logit_positive)/(exp(logit_positive)+exp(logit_negative))`与完整词表合法代码总质量mass；q仅为二代码条件概率，不能当自由回复语义概率。

## 主要比较、门与停止决定

以下是本轮**主动选择的工程门**，非正式AC2-16，不自动建议写入P5-31。0.95格式/质量、0.90 oracle、0.05–0.95动态区间参考R-CHOICE-02；本轮重新测。0.20布局范围及“每格至少一根”是执行者的本轮设计选择：避免极端代码偏好被总体平均掩盖；仅两根不能估计总体效应。这些数值不作用于她的状态、规律或生产行为。

1. 完整性：462/462完成，三语各自的两次完整logits字节相同；不一致或缺件不判可用。
2. 每语表达槽：argmax合法率≥0.95；至少0.95请求mass≥0.95。每语oracle：正确率≥0.90（INVALID算错），至少0.95请求mass≥0.95。保留四家族、映射和位置明细，不只给均值。
3. 每个**概念×语言×附语**格有2根；至少1根在8个映射/顺序/字母表布局中，q中位数严格介于0.05与0.95，且max(q)−min(q)≤0.20。24格全部满足才称C0_usable。逐格报告代码、字母表、位置、附语差异与地板/天花板，不过滤样本。
4. 24个自由回复原样交付，报EOS/上限、token数、思考标记及不完整UTF-8；不自动给概念或连贯分。独立评分not_run，不能以槽结果替代。
5. 任一门失败→C0_insufficient_stop，停止后续方向试验；不改题、换接口、调阈值或提取方向。全部通过→仅允许准备下一阶段，其完整样本和预注册再次推送后才能跑。通过不等于概念资格；不足不等于全路线证伪。

C0根全部列为已见校准，后续提取/选层开发/最终检验必须用全新场景、实体与句法根。不在C0选择方向、层、剂量。方向最小效应和能力下降容差须后继运行前另冻，不能把oracle正确率门偷换成正式能力无损。

## 资源与失败处理

预计 **5–10分钟GPU窗口，native硬上限10分钟**，含哈希准备申请15分钟。估计参照P0-12文本decode约116 token/s、R-CHOICE-02每576 prefill约36秒，模型/输入/观测开销不同，非性能承诺。请求60秒、加载60秒、正常退出30秒为工程超时；独立watchdog600秒只结束本轮启动的native进程树，不动背景进程，不自动重试。

用户确认窗口后，再确认Runner.Listener/Worker均无（无法确认也停）；记录全进程名/PID作为背景基线。Codex、浏览器和桌面常驻本身不算冲突，只排除重负载；入口拒绝runner、llama-server、Blender，其他推理/游戏/视频负载窗口前人工确认。测量时无编译、测试、下载转换或其他GPU工作。Python每请求、native每decode重查runner；不声称连续监测所有后台。结束通知恢复runner。

首个下载、构建或实测错误保留原始日志，不自行换工具或重跑失败测量。输出新建`D:/Dev/cache/R-STEER-01/c0-run-01`，不覆盖。沿P0-12 [archive_evidence.py](../../spikes/carrier-engine/archive_evidence.py)用SHA256→gzip objects/filemap无损归档到本报告目录，逐文件解压核验；包括失败、提示、源码/二进制绑定、protocol、输出及logits摘要，可离线复算。

无外部模型、付费API或载体自评。研究者编题/预标注存在偏差，后续须独立核对；模型的自述不算她的状态或成长证据。

## 复现入口（当前未执行GPU）

```powershell
pwsh -NoProfile -File reports/R-STEER-01/build.ps1 -OutputDirectory D:/Dev/cache/R-STEER-01/build-01
$env:PYTHONDONTWRITEBYTECODE='1'
& D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe reports/R-STEER-01/calibrate.py prepare --output D:/Dev/cache/R-STEER-01/c0-run-01
# 仅在用户确认本轮窗口且runner已停后：
& D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe reports/R-STEER-01/calibrate.py run --output D:/Dev/cache/R-STEER-01/c0-run-01 --executable D:/Dev/cache/R-STEER-01/build-01/calibration_probe.exe --quiet-window-confirmed
# GPU退出后离线分析（输出必须不存在）：
& D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe reports/R-STEER-01/analyze_c0.py D:/Dev/cache/R-STEER-01/c0-run-01 --output D:/Dev/cache/R-STEER-01/c0-summary-01.json
```
