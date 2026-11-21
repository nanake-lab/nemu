中档。交付线，本次修改 Python worker 的离线接口与测试，新增运行时依赖为零。

# P5-11 「乎」评估 worker

## 当前状态

**部分交付。** 文字和语音观察可以经过本地接口生成带原绑定的四维提议，测试使用合成输入。重复收到结果未知的执行回执，两次均返回「未支持」，后端调用次数为零。只有好奇一维有已知零时，结果只保留这一维，其余三维仍未知，整条记录不能进入有效 A。正式评分由 P5-11 在 DESIGN-APPRAISE-02 定下量尺后接续，三语质量验收保留在本卡后续。[整卡未完成项](BLOCKED.md)

## 交付与验收

| 项目 | 结果与证据 |
| --- | --- |
| 接口与绑定 | 使用 P5-21 的 NatureAppraisalRecord、NatureVector 和完整 ProposalBindingV2。快照在调用前固定，事件、已接纳 z、事实投影、历史前缀或资源错配时拒绝。回调仅能通过接口取得 z、c_core 和所涉人引用。[接口说明](../../workers/drive/README.md) · [规格及测试对照](../../docs/traceability/P5-11.md) |
| 用户指定的两个对照 | 均通过。未知回执从未进入回调。纯噪声夹具只提供好奇的已知零，未生成 NatureAppraisalRecord。四维全有值、全部未知、超界、非有限值、漏字段和多字段均有独立覆盖。[离线用例](../../workers/drive/tests/test_appraisal.py) |
| `uv run --locked pytest`、`ruff check .`、`mypy --strict src tests` | 21 项测试通过，包括 20 个接口用例和原有导入检查。250 ms 的真实时钟超时用例单独运行耗时 0.26 s，后端取消并完成清理。静态检查通过。首次试跑发现黄金绑定的读集为空，无法检验嵌套快照隔离。换成显式非空的合成读集后，该测试覆盖了调用期间修改读集与语义图的情形。 |
| `uv build`、独立环境安装 wheel | sdist 与 wheel 均构建成功。在仓库外以 `python -I` 载入已安装的包、IDL 类型和随包 schema，并对该 wheel 重跑同一组 21 项测试，全部通过。 |
| Python 提议交给 Rust 核心 | 将实际返回的提议和调用前独立保存的绑定以 CanonicalV1 交给 `core_runtime::binding::validate_record`。原记录通过。分别改动事件、分支、状态修订、z 摘要、事实摘要、评估资源版本、读集和载荷摘要，8 项均被拒绝。[核验程序与原始日志](https://github.com/nanake-lab/nemu-evidence/tree/269ac356a70eb2597210e23d0ca75a82391b5b53/P5-11/offline) · [证据索引及 SHA-256](evidence.json) |

测试桩位于 tests，原样返回合成数据。程序包只包含接口、验证和现有生成类型。无默认评估器，无权威存储写入。真实评估语义、语音识别质量、真实 m = 0、IPC 及降级写 H 均未在本次运行。[运行边界](../../workers/drive/README.md)

## 逐份核对

开工基线为 `da40fd067f5c659fe34b35eb03571ef5921fce08`。用户已确认先交付离线部分，并将量尺设计指定给 DESIGN-APPRAISE-02。

同步 main 到 `4a7b8a8b` 后，新增改动集中在独处演示及其文档。合并提交 `61d3bfbb` 上重跑了本包的测试、静态检查和构建。重建 wheel 与独立环境实际测试的 wheel 的 SHA-256 一致。PR 当前提交的托管与自托管结果见 [PR 检查](https://github.com/nanake-lab/nemu/pull/245/checks)。

| 依据 | 对 P5-11 输入、输出或验收的影响 |
| --- | --- |
| [ADR 0042 §4 U5/U6](../../docs/adr/0042-cause-projection.md#L178-L195) | U5 分配说话、独处与行动选择。U6 要求解释失败时如实显示故障，迟到的补解释不补发。派工指定「乎」评估只在本地、以 m = 0 运行。U5 文中的「本地 A」指载体路线，评估算子的读集依据仍是 EV6。 |
| [ADR 0043 §3、§5](../../docs/adr/0043-two-lines.md) | 载体研究和首版交付分别验收。本卡只需可替换的本地测试接口，真实端点联测另记；研究线的 GPU 测量不属于 worker 单测。 |
| [ADR 0045 D1、§2](../../docs/adr/0045-speak-think-wiring.md#L7-L24) | 过渡期云端只能生成独处念头文字。依本次派工收窄卡面的后端选择，移除远程日常档作为 A 的实现路径；不影响向量范围和三语质量标准。 |
| [ADR 0046 §2.1](../../docs/adr/0046-speak-think-contract-gaps.md) | 云端内部文字有单独的回执和采纳合同。P5-11 的成功输出沿用 P5-21 的 NatureAppraisalRecord，由语义核心治理。 |
| [ADR 0047 草案「背景与决定」第 3 项](https://github.com/nanake-lab/nemu/blob/59a79fab4ac2f62836a67965f2ea4ad61aacf0cc/docs/adr/0047-permission-contracts.md#L5-L15) | [#235](https://github.com/nanake-lab/nemu/pull/235) 尚为草稿。提案只约束授权、凭证和写权，未给 A 增加读取、执行或提交权限。本次没有采用草案接口。 |
| [证据合同 U1–U9](../DESIGN-EVIDENCE-CONTRACT-01/DECISIONS.md) | **影响验收。** 用户已选定首批仅支持文字和语音；U3 把字段到语义的适配限定为已冻结来源；U4 区分未评、已评未知、已知零和失败。对已支持且足以评价的输入，仍检查确定性和完整四维；对未覆盖或不足的输入，必须检查不会产出有效评分。U1/U2/U9 涉及来源、概念及版本，U5–U8 属下游证据处理。 |
| [时钟报告 §2](../DESIGN-CLOCK-01/REPORT.md#L15-L27) | U1–U7 涉及时间证明、提交、年龄、恢复与长时段学习。A 不从墙钟、离开时长的猜测或超时时间推算强度；接口超时仍返回失败。该组决定没有给 worker 新增时钟字段或质量阈值。 |
