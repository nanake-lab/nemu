# 登录项只读检查与 scaffold 保护（2026-10-24）

## 原有行为与修正

`origin/main`（0b5412e）的 `apps/desktop/src/main/index.ts` 已有无条件调用 `app.setLoginItemSettings({ openAtLogin: settings.general.openAtLogin })` 的行为；**这不是 P6-08 引入的**。调用也会出现在临时 scaffold 模式中。本卡按用户授权顺带修复测试路径：只增加 `if (!scratch)`，产品无 scratch 的原行为不变，不改默认值或设置 schema。

真实打包启动回归测试会记录 setLoginItemSettings 的调用次数并断言当前 scratch 组为 0。基线组仍拦截原调用、绝不透传到 OS。使用临时 userData，窗口不显示，感知源为合成数据；不拿用户设置作测试。

## 只读快照

时间：Perth 2026-10-24 16:51:42。原始证据 [startup-audit-20261024.json](evidence/startup-audit-20261024.json)。名称、注册表命令数据、启动文件名或快捷方式目标按大小写不敏感的 Electron/Nemu 匹配：

| 检查位置 | 匹配条目 | 可取得的写入时间 |
| --- | --- | --- |
| HKCU\Software\Microsoft\Windows\CurrentVersion\Run | 0 | 无逐值时间；整个 Run 键最后修改为 Perth **2026-10-23 10:24:00.015**（UTC 02:24:00.015） |
| 当前用户 Startup 文件夹 | 0 | 无匹配项，因此无条目时间 |
| 公共 Startup 文件夹 | 0 | 无匹配项，因此无条目时间 |

文件夹分别为 `C:\Users\nanake\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup` 与 `C:\ProgramData\Microsoft\Windows\Start Menu\Programs\Startup`。没有删除、写入注册表、保存/修改快捷方式，也没有执行启动脚本或读取其正文。注册表以只读权限打开；时间通过 RegQueryInfoKeyW 取得，其 FFI 仍位于唯一 win32.ts。

## 能与不能判断的事项

当前未发现上述匹配项。Run 键只提供**键级**最后修改时间，不能分配给某个值；当前条目不存在也不能证明历史上从未写入或删除过。这个快照**不能归因以前的 CI/e2e 是否执行过写入**，也不能把键最后修改时间当作 Nemu/Electron 条目写入时间。未扫描用户注册表事务历史，未推断其他应用的 Electron 技术栈。
