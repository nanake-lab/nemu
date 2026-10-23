# 第二 Windows 账户的拒绝连接验收（待用户安排）

本步骤尚未执行，不能算通过。无需创建账户或修改系统设置。

用户准备：现有第二账户，能读取本任务的代码及运行 Rust 构建结果；两个账户同时登录或可切换，
第一账户的探针保持运行。不要给第二账户授予管道权限或分享终端令牌。

两边的 PowerShell 都先指定各自可写的 **D 盘**临时目录/缓存（下面可按用户安排调整为另一个 D 盘目录）：

```powershell
$env:CARGO_HOME = 'D:\Dev\cache\cargo'
$env:CARGO_TARGET_DIR = 'D:\Dev\cache\cargo-target\P5-03'
$env:TEMP = 'D:\Dev\cache\P5-03\tmp'
$env:TMP = $env:TEMP
Set-Location D:\Dev\nemu-wt\P5-03\core
```

第一账户运行：

```powershell
cargo run -p gateway --example acl_probe --locked -- serve
```

它只用新建临时库绑定真实 ACL 的管道，显示 `\\.\pipe\nemu-core-<SID>`，窗口为 120 秒。
若已有核心占用管道，探针拒绝启动；不要停止或替换正在运行的用户核心来强行测试，改由用户安排窗口。

第二账户在窗口内运行（替换第一账户显示的 SID）：

```powershell
cargo run -p gateway --example acl_probe --locked -- deny <第一账户SID>
```

必须报告 `passed: different Windows SID received ERROR_ACCESS_DENIED (5)` 且退出码 0。
探针会检查当前 SID 确实不同。找不到管道、PIPE_BUSY（231）、超时及任何其他错误都不是通过。
第一账户保留启动窗口的日志，第二账户保留拒绝连接输出和退出码，交回本 PR 补入证据。
全程没有凭据交换，不打开已有排练库/正式库。探针不会自行执行第二账户登录或创建账户。
