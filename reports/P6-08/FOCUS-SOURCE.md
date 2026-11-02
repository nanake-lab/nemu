# P6-08 专注模式来源调查（2026-10-24）

本机只读系统信息：Microsoft Windows 10 家庭中文版，build 19045，10.0.19045。SDK 26100 的安装不表示 OS 支持 SDK 中的所有运行时类。

官方来源：

- [FocusSessionManager](https://learn.microsoft.com/en-us/uwp/api/windows.ui.shell.focussessionmanager?view=winrt-26100)：Windows 11 Insider Preview，最低 10.0.23504.0；读取路径为 IsSupported/GetDefault/IsFocusActive。
- [Detect and react to focus session state](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/applifecycle/focus-session)：先核对 IsSupported，再读取状态或订阅变化；这是 Windows 11 Focus session。
- [QUERY_USER_NOTIFICATION_STATE](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ne-shellapi-query_user_notification_state)：QUNS_BUSY 表示全屏/演示；QUNS_QUIET_TIME 是首次登录或升级后的系统 quiet time。文档没有把它定义为 Windows 10 Focus Assist 的可靠开关。

因此不能用这个 Windows 11 API直接满足本机 Windows 10 的 Focus Assist 验收，也不能用现有通知桩矩阵宣称专注开关已经验证。未读取/修改专注设置，未切换任何系统状态。

## 调查时提出的两条路线

1. 在本卡内允许 **仅只读 WNF 可行性探针**：保持单一 FFI 文件、不引依赖、不调用状态更新 API、不读窗口内容、不改设置；先验证本机返回结构与版本边界，再提交具体适配候选。WNF 是未文档化接口，不能预先保证适配稳定；未知或错误必须明确处理，不能当作“专注未开启”。预计先用 15–30 分钟核验可行性，若能实现，采样路径改变后需重新登记产物并按同一 CPU 方法验证。
2. 只用受支持公开接口：保留 Windows 10 自动专注识别为未支持，使用已有手动勿扰边界；这意味着不能满足卡片当前的完整专注自动检测验收，需要用户批准范围/验收变更（ADR），代理不能自行视为通过。

## 用户决定与探针结果

用户于 2026-10-24 批准路线 1，限定只读、已有 koffi、单一 FFI 文件；Windows 11 仅用公开 FocusSessionManager，Windows 10 才可用 WNF；未知 build、接口缺失/失败、格式不匹配均为 unknown，不冒充 inactive、不抛出异常。Windows 10 专注识别改为探针结果报告，**不作为本卡必过项**；全屏/演示/会议等其余来源仍按原验收。

实际结果：`10.0.19045`，NtQueryWnfStateData 返回 0，载荷 4 字节，rawValue=0，按社区状态映射为 inactive，changeStamp=32039。没有切换设置验证开/关，不声称识别正确率 100%。原始证据在 `evidence/focus-probe-19045-20261024.json/.txt`。

来源采用 [Alex Ionescu 的 wnfun 状态名表](https://github.com/ionescu007/wnfun/blob/master/script_python/WellKnownWnfNames.py) 及 [Rafael Rivera 原始示例](https://gist.github.com/riverar/980120d7e3a13ed8b1d665cf974c8e31) 的 ABI/常量事实：`WNF_SHEL_QUIETHOURS_ACTIVE_PROFILE_CHANGED = 0x0D83063EA3BF1C75`。接口未公开、未受支持，Windows 更新后可能失效。只核验 build 19045，不扩大白名单。

探针和注册表时间审计帮助函数不从公共包入口导出，没有接入正式采样/勿扰循环或核心事件。Windows 11 公开读分支仅有 SDK 对照和桩测试，本机没有 Windows 11 实测。新增探针后的正式主进程构建与原包逐字节一致，见 `evidence/focus-production-bundle-comparison-20261024.json`；之后的 scaffold 登录项保护只影响启动路径，不改变感知循环。

限定决定完整记入 [ADR 0023](../../docs/adr/0023-desktop-focus-probe.md)。未来接入自动勿扰策略前另行明确 unknown 的业务处理，并复验兼容性和 CPU。
