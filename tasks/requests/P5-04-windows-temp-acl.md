# P5-04：Windows D 盘临时目录 ACL 最小修复请求

## 证据与边界

用户指定手动运行 windows.yml；本机检查已完成，提交 785a1bb 的 Ubuntu 39983818418 成功。Windows 39986857254（2026-10-24 本地 00:27 手动触发）在 isolation 的 icacls /setintegritylevel '(OI)(CI)M' 返回 Access denied；W02/W19 未运行。旧 39901710241 同样失败。失败日志、运行元数据与原始 artifact 位于 reports/P5-04/main-sync-20261024/。

只读 ACL 检查：D:\Dev\cache\tmp owner 为 DESKTOP-8LBF84O\nanake，继承 SYSTEM/Administrators FullControl、Authenticated Users Modify、Users ReadAndExecute，没有当前用户的显式 FullControl。这支持最小 ACL 修复候选，但未实际尝试修复，不能声称已证明根因或通过。

AGENTS.md 第 3 节第 13 条：“同一个验收项认真试过一种方法仍失败……不再自行尝试替代方案……等用户决定”。因此先保留草稿和证据，等待以下方案的选择，不继续重试或改测试。

## 方案 1（推荐，请用户授权）

- 仅创建任务独占 D:\Dev\cache\tmp\P5-04-ci；所有操作先检查绝对路径与该精确目录一致。
- 在该目录保留原 SYSTEM/Administrators 和其他已有权限，只为实际 runner 用户 SID 增加可继承的 FullControl。记录修改前后 ACL；不改 D:\Dev\cache\tmp 根或别的缓存目录的 ACL，不申请提升、不改 runner 服务/账户/完整性级别。
- 仅修改本卡已经追加的 .github/workflows/windows.yml TEMP/TMP 值为该目录，并在最前追加创建目录/核对 runner SID/设置该目录 ACL 的步骤，使 CI 可重建该环境。其他现有条目不改。
- 先用同一个现有隔离 smoke 验证，再触发相同 Windows 工作流；不改 test-windows-isolation.ps1、不跳过检查、不改低完整性子进程权限或验收阈值，继续使用 D 盘临时数据。
- 若同一修复仍失败，停止该验收并带证据回报，不继续其他 ACL/运行身份替代方案。

## 方案 2

用户自行修复 runner 的 D 盘临时目录权限；代理保留 draft + needs-user，收到恢复指示后再跑完整 Windows。无代理 ACL 变更，代价为继续等待环境修复。

## 不涉及

不修改 core-store、core-types、IDL、冻结向量、模型或任何数据存储；不打开排练库/正式库，不改全盘 ACL，不切换 C 盘，不联系其他执行者、不发审查请求。
