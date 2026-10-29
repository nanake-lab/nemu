# 运行说明（当前只完成离线准备）

先读 [PREREG](PREREG.md)、[COSTS](COSTS.md)、[BLOCKED](BLOCKED.md)。本目录是研究代码，不是生产入口；只能运行C组。所有人物与历史均为虚构。当前没有API/GPU结果。

## 文件与环境

- `worlds.py`：配对事件、四名人物、原始事实检索、能力题和标签布局。
- `run.py`：离线生成设计；获批后执行C组行为阶段。
- `carrier.cpp` / `carrier.py` / `build.ps1`：ADR0031原生零调制探针；每请求清KV、m=0、全词表单动作槽。
- `io_support.py`：仅DeepSeek人物API的独占账本/环境密钥/固定地址/禁止重试；不再含Anthropic或付费judge路径。
- `judging.py` / `evaluate.py` / `codex_judge.py`：盲评包、16次主评判、负对照、锚点及人物审计；可离线导出新匿名目录。订阅Codex的严格隔离/实际版本/条款未核实，live入口硬阻断。
- `probability_analysis.py`：直接从原始全词表概率计算H1a，独立于任何judge/API。
- `planning.py`：10项H1族、D排除、预注册随机评判子样本、解析功效与DeepSeek/订阅会话预算。
- `analysis.py` / `diagnostics.py`：种子级分析、功效规划、标签偏差/动态范围/能力指标。
- `test_research.py` / `test_revision.py`：离线协议核验。测试夹具不是行为证据，结构性零夹具不是运行D组。

使用Windows PowerShell、已有Python 3.12标准库、已有MSVC14.44。无需新增Python运行时包。类型/静态检查使用已有缓存工具，不安装依赖。

本机只读复用 `D:/Dev/cache/P0-10/` 的固定头文件、llama.cpp DLL与Q4模型；缺失或哈希不符时停止，不下载、不换模型。编译器输出及临时文件放 `D:/Dev/cache/R-LIFE-01/`。仓库中的 `spikes/carrier-engine/native_probe.hpp` 是只读研究探针支持代码。

## 离线核验与准备

在工作树根目录执行。下列命令不调用API、不加载GPU模型；输出目录必须是新的，已有证据不覆盖：

```powershell
$env:PYTHONDONTWRITEBYTECODE = '1'
$env:PYTHONUTF8 = '1'
$researchPython = 'D:/Dev/cache/uv/archive-v0/zZzYXHvD_8D046l3/Scripts/python.exe'
& $researchPython -m unittest discover -s reports/R-LIFE-01 -p 'test_*.py' -v
& $researchPython -m mypy --strict --cache-dir D:/Dev/cache/R-LIFE-01/new-checks/mypy reports/R-LIFE-01
& $researchPython -m ruff check --cache-dir D:/Dev/cache/R-LIFE-01/new-checks/ruff reports/R-LIFE-01
& $researchPython reports/R-LIFE-01/run.py prepare --output D:/Dev/cache/R-LIFE-01/new-preparation
```

`prepare`保存不变的世界清单、当前协议/代码哈希、`analysis-plan.json`（功效、候选抽样种子与预算）和`not_run`状态。原`prepared-01`保留v1证据，不覆盖。它不生成载体行为，不代表试点通过。

编译前检查重负载窗口，单独执行 `build.ps1`。本轮已编译，`build-01`不可覆盖。编译输出`build.json`记录内部编译退出码；外层终端是否结束单独报告。

```powershell
& reports/R-LIFE-01/build.ps1 -OutputDirectory D:/Dev/cache/R-LIFE-01/new-build
```

## 获批之后的执行顺序（本轮尚未执行）

1. 本轮用户只授权离线修订。将来运行须记录本卡DeepSeek累计费用批准（v2提案≤$0.50，原$30作废）并重新核价；检查已付/未知费用。不得把新目录当新额度。当前累计0。验证本版协议/代码绑定；不可与旧版本混用。Codex启动还依赖[CODEX-JUDGE](CODEX-JUDGE.md)的核验，不以费用批准替代。
2. 唯一API密钥为用户提供的进程环境`DEEPSEEK_API_KEY`。Codex只使用已有ChatGPT登录，不读取/复制认证内容，不用Anthropic/OpenAI API key。不要将密钥写进聊天、文件或设置。
3. 明确本轮GPU安静窗口，用户停runner。确认`Runner.Listener`、`Runner.Worker`均不存在；无法确认则停。窗口内只运行本研究，首个最小请求失败即停止，最长90分钟。
4. 选择全新运行目录，并将下面的金额/授权引用占位符替换为真实已获批记录。CLI参数只是操作留痕，不能创造授权：

```powershell
$protocolSha = (Get-FileHash reports/R-LIFE-01/PREREG.md -Algorithm SHA256).Hash.ToLowerInvariant()
# 仅在费用和本轮窗口均已获批后执行；金额不能超过本卡剩余额度。
& $researchPython reports/R-LIFE-01/run.py live `
  --output D:/Dev/cache/R-LIFE-01/approved-C-run `
  --executable D:/Dev/cache/R-LIFE-01/build-01/carrier.exe `
  --approved-usd '<已批准金额>' --approval-ref '<批准消息引用>' `
  --prereg-sha256 $protocolSha --quiet-window-confirmed
```

5. 正常/异常退出后确认本研究探针结束，通知用户恢复runner。保留全部原始文件；不自动换接口、恢复半个人生或补种子。
6. 行为阶段全部完成后，先用`probability_analysis.py`独立计算H1a（额外API/GPU为0）。它不以评判成功作为入口，结果在全量语义审计前仍是provisional。再用`evaluate.py --mode prepare-jobs`离线导出688个新目录，每目录只有当前匿名材料；父目录私有真值清单不能进入judge可读范围。目录布局本身不是沙箱证明。

```powershell
& $researchPython reports/R-LIFE-01/probability_analysis.py D:/Dev/cache/R-LIFE-01/approved-C-run
& $researchPython reports/R-LIFE-01/evaluate.py D:/Dev/cache/R-LIFE-01/approved-C-run --mode prepare-jobs
```

7. 当前`--mode run`在Codex适配器处无条件阻断，不启动进程，不会退回API；必须先解决严格能力及条款缺口。未来每次需新exec/新thread、可信实际模型版本回执、无工具/越界证明，变版本或额度不足停止。`check_events`/`check_model_receipt`只是必要的导入核对，不代表CLI已提供这些证据，不能通过手填布尔值开跑。
8. 主代理逐条核对人物原话/全部载体prompt，核验原文与时间、检索≤4、无人格文本；按事件号标反应类别、核对judge依据。自动统计不替代语义核验或自动宣布通过。主分化可独立于judge解释报告，但不豁免输入合规与能力门。

## 已知边界

原生探针编译通过不等于GPU运行通过。DeepSeek接线未实测；Codex严格能力/模型身份/条款未核实并硬阻断。相同模型别名不能证明后端不变；版本来源无法核验不能接受。费用预留不代替供应商账单，订阅额度不等于无限免费请求。

本地包装核验模型、`llama.dll`及实际探针哈希；P0-10其他运行时DLL仍依赖既有只读缓存，正式运行前应归档完整DLL清单。四人/单领域/短历程、单token选择与12条规则能力题均限制外推，不能替代完整核心、开放对话、长期个体或AC2-16验收。
