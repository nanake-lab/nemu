# 运行说明：C两阶段，当前仅离线准备

先读[PREREG](PREREG.md)、[PHASE1](PHASE1.md)、[COSTS](COSTS.md)、[BLOCKED](BLOCKED.md)。只有C组运行入口，所有人物与历史虚构，临时EXPERIMENT目录；尚无模型/API/GPU结果。订阅Codex已停用，`codex_judge.py`/CODEX-JUDGE仅作历史记录，不属于现行执行链。

## 文件和环境

- `worlds.py`：原有世界、人物、原始事实检索、能力题、标签布局，本轮未改。
- `run.py`：离线prepare或已获批的阶段一人物/载体运行；无judge凭证/选型前置。
- `carrier.cpp`/`carrier.py`/`build.ps1`：固定ADR0031 Q4零调制载体，每请求清KV，未改、不重编译。
- `io_support.py`：仅DeepSeek人物的环境密钥、预算账本、禁止重试。
- `phases.py`/`PHASE1.md`：阶段一协议与代码独立绑定、完成时冻结数据；阶段二只选judge不会改写它。
- `phase1.py`：原始历史/全部输入重建、H1a、动态范围/动作/能力、重复率，生成需人工完成的原文审计模板；不访问judge。
- `probability_analysis.py`/`analysis.py`/`diagnostics.py`：原始概率、种子级统计和质量诊断；不把未知/未运行当通过。
- `anonymous_jobs.py`/`judging.py`/`evaluate.py`：阶段二独立匿名包、主配对/锚点/负对照/人物审计；门槛不变。
- `hosted_judge.py`：GLM-4.6、MiniMax-M3的固定无状态chat接口；无默认厂商，单独审批和预算；未做真实API验证。
- `planning.py`：原10项H1族、D排除、预注册子样本和解析功效，新增两家费用比较。
- 三个`test_*.py`：离线夹具；固定数字/固定原话不属于C/D或模拟人物模型结果。

Windows PowerShell，已有Python标准库；不安装运行时依赖，不下载模型，不修改用户/系统设置。Python、MSVC和P0-10缓存仍使用已有版本。本地模型与llama.dll/hash不符即停。模型、原生探针完整DLL清单需在将来运行时归档。

## 离线检查

工作目录为此worktree根；产物目录必须新建，不能覆盖旧证据：

```powershell
$env:PYTHONDONTWRITEBYTECODE = '1'
$env:PYTHONUTF8 = '1'
$researchPython = 'D:/Dev/cache/uv/archive-v0/zZzYXHvD_8D046l3/Scripts/python.exe'
& $researchPython -m unittest discover -s reports/R-LIFE-01 -p 'test_*.py' -v
& $researchPython -m mypy --strict --cache-dir D:/Dev/cache/R-LIFE-01/new-checks/mypy reports/R-LIFE-01
& $researchPython -m ruff check --cache-dir D:/Dev/cache/R-LIFE-01/new-checks/ruff reports/R-LIFE-01
& $researchPython reports/R-LIFE-01/run.py prepare --output D:/Dev/cache/R-LIFE-01/new-preparation
```

prepare保存世界设计、完整源哈希、阶段一独立哈希、功效/预算计划及not_run状态，不加载载体、不生成行为。编译已有v1证据沿用，本轮不运行build.ps1或任何GPU程序。

## 阶段一：无需评判者

1. 先取得用户对阶段一协议与本卡DeepSeek累计金额的明确批准（提案≤$0.50），复核价格、已有/未知计费扣除。本轮用户禁止实际调用；以下只说明未来用法。
2. 用户仅以进程环境提供`DEEPSEEK_API_KEY`，不写聊天/文件/仓库/设置。无需评判者key、订阅登录或judge选择。
3. GPU另需用户安排本次安静窗口；用户停runner，运行前/每请求检查Listener和Worker，不能确认则停。20–60分钟规划，90分钟硬限；首个冒烟失败即停，不换模型重试。
4. 使用全新目录与真实已批准记录；参数本身不产生授权：

```powershell
$protocolSha = (Get-FileHash reports/R-LIFE-01/PREREG.md -Algorithm SHA256).Hash.ToLowerInvariant()
& $researchPython reports/R-LIFE-01/run.py live `
  --output D:/Dev/cache/R-LIFE-01/approved-C-run `
  --executable D:/Dev/cache/R-LIFE-01/build-01/carrier.exe `
  --approved-usd '<已批准的本卡剩余额度>' --approval-ref '<用户批准引用>' `
  --prereg-sha256 $protocolSha --quiet-window-confirmed
```

5. 正常/异常结束均确认探针关闭，通知用户恢复runner。不补种子、不恢复半人生；完整结束才冻结数据并写阶段一完成状态。
6. 以下仅CPU，生成不依赖judge的结果及**未完成**审计模板：

```powershell
& $researchPython reports/R-LIFE-01/phase1.py D:/Dev/cache/R-LIFE-01/approved-C-run --review-template
```

主代理逐条核对全部原话/实际prompt/来源时间/原文条数/人格文字/事实矛盾。复制模板为新`phase1-local-review.json`后填写审阅者、核对声明和384条带事件号的反应类别，可多标签；保留模板及所有原始数据。四个稳定人物岗位分别统计≥3类反应，姓名在种子内对应岗位。自动结构/正则检查不能代替语义审阅；缺审阅时门值null、不会判通过。

```powershell
& $researchPython reports/R-LIFE-01/phase1.py D:/Dev/cache/R-LIFE-01/approved-C-run `
  --review D:/Dev/cache/R-LIFE-01/approved-C-run/phase1-local-review.json `
  --output-name phase1-reviewed-summary.json
```

阶段一H1a采用未观测H1b暂置p=1的两项Holm保守值，完整H1族不缩小。阶段一满足不代表自然度/H1b/完整试点通过；H1a不显著也不自动阻止独立报告H1b。后续judge选型不改阶段一冻结单元，改变它则需新版本。

## 阶段二：先由用户选厂商、版本和单独上限

[COSTS](COSTS.md)提供GLM或MiniMax两方案；MiniMax有实名条款，Kimi价格未核实。用户明确厂商/版本ID粒度、当前协议与累计上限后才可运行。若要求不可变build回执而供应商无法提供，则继续阻塞。账号/支付条件由用户处理，代理不代注册或提交身份资料。仅用户提供`ZAI_API_KEY`或`MINIMAX_API_KEY`，禁止其他API或订阅回退。

可以先离线导出匿名包（不会调用API）：

```powershell
& $researchPython reports/R-LIFE-01/evaluate.py D:/Dev/cache/R-LIFE-01/approved-C-run --mode prepare-jobs
```

`judge-jobs`仅每次匿名材料；`judge-private-manifest.json`保存真值，不能发送。请求体只有system+当前材料，没有路径/组别/种子/真值/上一回答。厂商只能通过HTTP取得显式材料，无本机文件系统或工具宿主。当前适配器未进行真实响应验证，不能把单元夹具当兼容性证明。

将来正式启动需所有参数，下面不替用户选择provider或授权金额：

```powershell
$judgeProtocolSha = (Get-FileHash reports/R-LIFE-01/PREREG.md -Algorithm SHA256).Hash.ToLowerInvariant()
& $researchPython reports/R-LIFE-01/evaluate.py D:/Dev/cache/R-LIFE-01/approved-C-run --mode run `
  --provider '<用户选择的glm或minimax>' --approved-usd '<已批准的阶段二剩余额度>' `
  --approval-ref '<独立评判批准引用>' --prereg-sha256 $judgeProtocolSha `
  --local-review D:/Dev/cache/R-LIFE-01/approved-C-run/phase1-local-review.json
```

入口先核对阶段一代码/数据绑定并复算本地审计和质量门，不要求H1a显著；再建立不可覆盖的judge审批文件。各请求model缺失/变化、工具、额外推理、超长、截断、错误、超时、用量异常均停；预留费用不退、不重试、不改写不合格回复、不补样本、不换模型。完整配对Holm及原所有校准门保持；阶段二发现泄漏会推翻相应人生的有效性，原H1a数值保留并标无效。

## 边界

本研究仍是四人/社区/短时/单动作槽和12条规则能力题，不能代替完整核心、长期个体、开放对话或AC2-16验收。正式32轮窗口/接口/核心依赖/非零方向另见[R-LIFE-02-CONDITIONS](R-LIFE-02-CONDITIONS.md)。本卡收尾只交审查，不合并、不领取下一卡。
