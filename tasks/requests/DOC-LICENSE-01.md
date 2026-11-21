# DOC-LICENSE-01：仓库采用 Apache-2.0

仓库：GitHub `nanake-lab/nemu`。开始前先通读 `AGENTS.md`。

**投入：** token 预算不设上限，不必节省。要读的文件全部读完，要查的情况全部查到，证据全部给出。不能为了少花 token 跳过步骤或简化检查。

- **线：** 交付线（文档，轻档）
- **分支：** `task/DOC-LICENSE-01`，从 `origin/main` 新建
- **目录：** 新建 `LICENSE`、`NOTICE`；`README.md` 只新增一节「License」；`tasks/requests/DOC-LICENSE-01.md`（本卡原文）；`reports/DOC-LICENSE-01/REPORT.md`
- **不碰：** 其他所有文件。不改 git 历史，不动 `content` 子模块。

## 用户决定

用户 2026-11-21 的原话：「OK 我们先加上 我选宽松」。Claude 推荐 Apache-2.0，原因是它宽松，并且带专利授权条款。

## 要做的

1. **`LICENSE`：** 放 Apache License 2.0 的官方全文，从 https://www.apache.org/licenses/LICENSE-2.0.txt 原样取来，一个字都不改。核对 SHA-256，与官方文本一致。
2. **`NOTICE`：**
   ```
   NEMU
   Copyright 2026 nanake-lab

   This product is licensed under the Apache License, Version 2.0 (see LICENSE).
   The `content` submodule (nemu-content) is a separate repository and is not covered by this license.
   Third-party models, motions and other assets keep their own licenses as recorded in their manifest.json files.
   ```
3. **`README.md` 的「License」一节**，中英各写一段，意思如下：
   - 本仓库的代码与文档采用 Apache-2.0；
   - `content` 子模块（她的角色设定、外观、声音等）是独立的仓库，不在此许可之内；
   - 第三方模型、动作等资产各自沿用 `manifest.json` 中登记的许可证。
4. **排查第三方材料**（只排查，不改动）：
   - 搜索仓库中所有 `manifest.json`，以及带有他人版权声明或许可证头的文件（`Copyright`、`SPDX-License-Identifier`、`License` 等关键词），重点看 `apps/stage/public/motions/`、`core/crates/fact-view/models/`、`packages/voice/`、`spikes/`、`tools/`。
   - 列一张表：路径、来源、许可证、能否随本仓库分发。
   - 发现以下任一情况，在报告里标红，并写「请 Claude 处理」：没有登记许可证的第三方文件；GPL 或 AGPL 代码；许可证不允许再分发的文件。
5. 检查：`git diff --check` 通过；README 里的链接都能打开；`LICENSE` 的哈希与官方一致。

## 交付

- 报告开头三行白话。第一行写：仓库从此采用 Apache-2.0，别人可以使用、修改，也可以商用，但必须保留版权和许可声明；`content` 不在此列。
- 开 draft PR，标题 `docs(DOC-LICENSE-01): 采用 Apache-2.0`，加 `needs-user` 标签，写「请审查」后停下。
