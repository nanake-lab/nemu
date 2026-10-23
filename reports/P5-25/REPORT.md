# P5-25 动力学 C：θ 的学习与回应回路 — 报告

## 当前状态

进行中。云端执行条件检查已完成，尚未实现 C，也尚未运行 P5-25 验收。保持草稿，完成后请求交叉审查；不自行合并。

### 云端试点记录

- 基线：main `0b5412e95caab447c70148c473e544dbab28c57c`。
- 环境：Linux 云端；Node v24.19.0、pnpm 12.11.2、uv 0.12.15、Python 3.12.14、Rust 1.99.0（含 clippy/rustfmt）。pnpm/uv/Rust 安装在用户目录。
- 源码取得：终端直接 Git 无认证；通过现有授权 GitHub 连接器按固定提交取得全部非 reports 目录文件和 reports 下代码，共 1,761 个文件，逐文件核对 Git blob SHA-1 与字节数。历史报告中的非代码证据未整体下载，按需读取。此副本是源码快照，不宣称完成 Git clone 或获得历史对象。一次连接中断在同接口重试后恢复，失败记录保留。
- 发布途径：现有 GitHub 授权接口创建任务分支、提交与草稿 PR，不向终端提供 token 或密钥。
- 已通过：用户目录工具安装、uv python install 3.12、uv sync --locked --project core/contracts、pnpm install --frozen-lockfile。
- content 子模块未初始化；未读取用户数据、排练或正式谱系存储。本机 D 盘与 Windows runner 条款按用户本次指示不适用。
- 已读 PR #73 head `25e8474958c7ccd9d0d37266a881c8df733f7058` 关于 θ 的提醒：连续反应似然、独立反馈稳定性检验、保留变化率下限；不直接搬用 R 的二元后验。
- 原始安装与后续测试证据将在本目录提交。尚未完成项目验收，不以环境检查替代验收。

## 交付物

待实现：`core/crates/core-kernel/src/dynamics/`；临时结构与契约请求遵循本次授权，不修改 core/contracts。

## 验收

| 检查 | 状态 |
| --- | --- |
| AC2-14 / AC2-05 θ / AC2-09 θ / I13 | not_run |
| AC2-25 性质部分 / THETA-1 / THETA-2 | not_run |
| Rust fmt / clippy / workspace tests | not_run |

## 待用户确认

新参考写法与未闭合合同的编号使用“待定”，在后续提交中逐项列明。
