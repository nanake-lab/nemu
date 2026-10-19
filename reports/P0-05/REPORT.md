# P0-05 仓库创建与环境安装 — 报告

## 结果

完成。私有 `nemu-content` 已包含模板和可追溯迁入的 Bible 历史；主仓库 `content/` 已替换为指向远端 `main` 的子模块。用户的 `.vroid` 和原始 `.vrm` 只保留在本机工作树，未提交到任一仓库。

## 交付物

- `https://github.com/nanake-lab/nemu-content`：私有内容仓库，当前 main 为 `bf6d4f1b01073d38a39cd4c6fe665f3068be546b`。
- `.gitmodules` 与 `content` gitlink：主仓库子模块 URL 和固定提交。
- `bible-verification.tsv`：迁移前后的 11 个 Bible 文件大小、SHA-256 与 Git blob 对照。
- `content-template-hashes.tsv`：模板文件哈希清单。
- `source-assets.tsv`、`migration-verification.txt`：用户源资产清单、备份及本次逐项核验结果。

## 验收

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| 迁移历史 | 通过；保留两条 P2-01 提交的作者、时间、说明和顺序；因目录改名重写 commit ID | 下方历史映射；`git -C content log -- nemu/bible` |
| Bible 文件 SHA-256 | 11/11 一致 | `bible-verification.tsv`；远端新克隆复核 |
| `content-template/` 文件哈希 | `.gitattributes` 之外 16/16 一致 | `content-template-hashes.tsv` |
| LFS 与历史 PNG | 远端新克隆中 LFS 对象数 0；5 张历史 PNG 按原字节保存，未来 PNG 仍走 LFS | `content/.gitattributes`；`migration-verification.txt` |
| 子模块远端与固定提交 | 通过；gitlink、远端 `main`、新克隆 HEAD 均为 `bf6d4f1b01073d38a39cd4c6fe665f3068be546b` | `git submodule status`；`migration-verification.txt` |
| 用户源资产保护 | 通过；两份源文件恢复到 `content/nemu/source/`，哈希匹配迁移前清单，未纳入提交 | `source-assets.tsv`；`migration-verification.txt` |
| P0-04 环境单项检查 | Blender 5.2.2 LTS、VRM Add-on、Office COM、Torch CUDA、Low IL 均 PASS；CUDA Toolkit 为 OPTIONAL | `../P0-04/AMENDMENT-2026-10-19.md` 与 `../P0-04/probes/` |
| Windows runner / CI | PASS；PR #21 的 Ubuntu 与 Windows 检查均成功 | [PR #21](https://github.com/nanake-lab/nemu/pull/21)，[run 39153136657](https://github.com/nanake-lab/nemu/actions/runs/39153136657) |
| 全量 doctor | 未重跑；遵从用户关于避免全量重跑、逐项隔离检查的指示。历史全量输出保留在 `reports/P0-04/doctor.txt`，修复后单项证据在 `reports/P0-04/probes/` | `../P0-04/doctor.txt`、`../P0-04/AMENDMENT-2026-10-19.md` |

### Bible 历史映射

| 原提交 | 内容仓提交 | 原提交说明 |
| --- | --- | --- |
| `67d788c640f4592ab7a372a1037424b2f336bfb6` | `eb3c8ce8da3f35f49db9a27c87f96cb5548c1330` | P2-01 初始人设种子 |
| `ae803df8e70b4cfb759a910e1fc8c96c8ba28197` | `77f5409920976f62a7dbfcb634e43d5e052aa38e` | P2-01 补充与校准 |

## 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| 无新增运行时依赖 | — | — | `git-filter-repo` 仅通过临时 `uvx` 执行历史迁移，未加入项目依赖 |

## 待用户确认

- [ ] （不阻塞）确认新 PR 合并后继续取 P2-03 资产管线卡；当前依赖 P0-05 第二部分和 P2-04 已满足。

## 后续建议（不在本任务范围内）

- 本地源资产仍未跟踪且不随子模块 clone 分发；由用户决定后续是否将其纳入私有内容仓。
- CUDA Toolkit 仅在项目新增本机 CUDA 扩展编译步骤时安装。

## 下一步

等待本分支 PR 的 CI 与合并；之后按任务顺序开始 P2-03。用户本机 `content/nemu/source/` 文件未加入任何提交。
