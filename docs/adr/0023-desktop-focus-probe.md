# ADR 0023：桌面专注状态的只读可行性探针

- 状态：用户已于 2026-10-24 明确批准下列限定范围，随本 PR 合并归档；不授权扩展成自动勿扰策略。
- 任务：P6-08；不改跨进程契约或 Echo Core 语义。

## 背景

实测主机为 Windows 10 家庭中文版 10.0.19045。SHQueryUserNotificationState 的官方合同没有保证反映 Windows 10 Focus Assist 开关；Windows.UI.Shell.FocusSessionManager 的官方最低版本是 Windows 11 build 23504。安装 SDK 26100 不代表当前 OS 具有该运行时类。

## 决定（用户限定授权）

1. 只交付只读可行性探针，沿用 koffi，不新增依赖。全部 FFI 在 `packages/desktop-sense/src/win32.ts`；不读取窗口内容，不写状态、不改设置。
2. Windows 11 分支仅用公开 FocusSessionManager（IsSupported/GetDefault/IsFocusActive）；不回退 WNF。低于其公开最低版本或查询失败为 unknown。
3. Windows 10 仅对本次确认的 build 19045 尝试 NtQueryWnfStateData，查询 `WNF_SHEL_QUIETHOURS_ACTIVE_PROFILE_CHANGED = 0x0D83063EA3BF1C75`。只接受 NTSTATUS=0、长度恰为 4、数值 0/1/2；未知 build、失败、缺 API、错误格式均返回 unknown，不抛出错误或崩溃，不把 unknown 当作 inactive。
4. **Windows 10 专注识别不作为本卡必过项，只报告探针结果。** 全屏、演示、会议等其余勿扰验收照常执行。
5. 探针未从公共包入口导出，未接入正式 source.sample/勿扰循环、AppBus 或核心事件；本卡不默默新增未知信号的业务门控策略。探针不改变已测 CPU 运行路径。

## 来源与实测

- 公开路线：[Microsoft FocusSessionManager](https://learn.microsoft.com/en-us/uwp/api/windows.ui.shell.focussessionmanager?view=winrt-26100)，ABI 取本地 Windows SDK 10.0.26100.0 的 windows.ui.shell.h，IFocusSessionManagerStatics IID `834df764-cb9a-5d0a-aa9f-73df4f249395`。
- WNF 名称来源：[Alex Ionescu 的 wnfun 状态名表](https://github.com/ionescu007/wnfun/blob/master/script_python/WellKnownWnfNames.py)；查询签名与 0/1/2 映射见 [Rafael Rivera 原始可行性示例](https://gist.github.com/riverar/980120d7e3a13ed8b1d665cf974c8e31)。只采用 ABI/常量事实，未复制第三方实现。
- 本机只读探针：10.0.19045，状态码 0，4 字节，值 0，映射 inactive；没有通过切换设置验证 on/off。证据为 reports/P6-08/evidence/focus-probe-19045-20261024.json。
- Windows 11 分支只有 SDK 对照与桩测试，未声称在 Windows 11 主机实测。

## 备选方案

- 用 SHQueryUserNotificationState 的 busy/quiet-time 冒充专注开关：未采用，文档语义不同。
- 修改注册表、启动/结束 Focus session 来验证：未采用，超出只读授权。
- 所有 Windows 版本均用 WNF：未采用，违反 Windows 11 优先公开 API 的决定。

## 后果

WNF 是未公开、未受支持的接口，Windows 更新后可能失效；这不是稳定性承诺。新增 build 需要先核验并登记，不扩大当前白名单。未来若把探针接入正式勿扰循环，需要另行明确 unknown 策略、复验隐私及采样 CPU，并保留本次来源/兼容性限制。P7 发布检查应包含 API 缺失/失效的降级验证。
