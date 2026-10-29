# 运行说明（当前只完成离线准备）

先读 [PREREG](PREREG.md)、[COSTS](COSTS.md)、[BLOCKED](BLOCKED.md)。本目录是研究代码，不是生产入口；只能运行C组。所有人物与历史均为虚构。当前没有API/GPU结果。

## 文件与环境

- `worlds.py`：配对事件、四名人物、原始事实检索、能力题和标签布局。
- `run.py`：离线生成设计；获批后执行C组行为阶段。
- `carrier.cpp` / `carrier.py` / `build.ps1`：ADR0031原生零调制探针；每请求清KV、m=0、全词表单动作槽。
- `io_support.py`：独占写入、费用预留账本、环境密钥、固定API地址、禁止重试。
- `judging.py` / `evaluate.py`：盲评包、16次主评判、负对照、锚点及人物审计。
- `analysis.py` / `diagnostics.py`：种子级分析、功效规划、标签偏差/动态范围/能力指标。
- `test_research.py`：离线协议核验。测试夹具不是行为证据。

使用Windows PowerShell、已有Python 3.12标准库、已有MSVC14.44。无需新增Python运行时包。类型/静态检查使用已有缓存工具，不安装依赖。

本机只读复用 `D:/Dev/cache/P0-10/` 的固定头文件、llama.cpp DLL与Q4模型；缺失或哈希不符时停止，不下载、不换模型。编译器输出及临时文件放 `D:/Dev/cache/R-LIFE-01/`。仓库中的 `spikes/carrier-engine/native_probe.hpp` 是只读研究探针支持代码。

## 离线核验与准备

在工作树根目录执行。下列命令不调用API、不加载GPU模型；输出目录必须是新的，已有证据不覆盖：

```powershell
$env:PYTHONDONTWRITEBYTECODE = '1'
$env:PYTHONUTF8 = '1'
$researchPython = 'D:/Dev/cache/uv/archive-v0/zZzYXHvD_8D046l3/Scripts/python.exe'
& $researchPython -m unittest discover -s reports/R-LIFE-01 -p test_research.py -v
& $researchPython -m mypy --strict --cache-dir D:/Dev/cache/R-LIFE-01/new-checks/mypy reports/R-LIFE-01
& $researchPython -m ruff check --cache-dir D:/Dev/cache/R-LIFE-01/new-checks/ruff reports/R-LIFE-01
& $researchPython reports/R-LIFE-01/run.py prepare --output D:/Dev/cache/R-LIFE-01/new-preparation
```

`prepare`保存世界清单、协议/代码哈希、解析功效和`not_run`状态。它不生成载体行为，不代表试点通过。

编译前检查重负载窗口，单独执行 `build.ps1`。本轮已编译，`build-01`不可覆盖。编译输出`build.json`记录内部编译退出码；外层终端是否结束单独报告。

```powershell
& reports/R-LIFE-01/build.ps1 -OutputDirectory D:/Dev/cache/R-LIFE-01/new-build
```

## 获批之后的执行顺序（本轮尚未执行）

1. 记录用户批准的**本卡累计**美元上限与该消息引用，重新核价；检查已有已付/未知费用。禁止把新目录当新额度。当前累计0。验证本版预注册哈希与代码绑定；代码更改不与旧运行混用。
2. 密钥仅由用户提供到执行进程环境：`DEEPSEEK_API_KEY`、`ANTHROPIC_API_KEY`。只检查是否存在，不打印值、不写配置、不持久化设置；不要发在聊天里。
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
6. 行为阶段全部完成后，使用同一目录、同一批准金额与引用执行`evaluate.py`；评判不占GPU。该阶段继续累加同一费用账本，不能重复启动。主配对仅向独立judge发送匿名经历与行为，真值在本地。

```powershell
& $researchPython reports/R-LIFE-01/evaluate.py D:/Dev/cache/R-LIFE-01/approved-C-run `
  --approved-usd '<同一已批准金额>' --approval-ref '<同一批准消息引用>'
```

7. 主代理逐条核对人物原话/全部载体prompt，核验原文与时间、检索≤4、无人格文本；按原始事件编号标注反应类别、核对judge依据。自动统计不会代替语义核验或自动宣布通过。将完整请求/响应、使用量、引用、种子、哈希与失败证据归档；只归档合成研究数据。

## 已知边界

原生探针编译通过不等于GPU运行通过。API接线尚未实测。云端相同模型ID仍可能有不可见后端漂移；返回模型字段检查只能发现可见变化。费用预留是保守软件门控，不是供应商账户硬限额；异常计费需人工核对。

本地包装核验模型、`llama.dll`及实际探针哈希；P0-10其他运行时DLL仍依赖既有只读缓存，正式运行前应归档完整DLL清单。四人/单领域/短历程、单token选择与12条规则能力题均限制外推，不能替代完整核心、开放对话、长期个体或AC2-16验收。
