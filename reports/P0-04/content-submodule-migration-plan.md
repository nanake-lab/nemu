# content/nemu/bible 迁移方案（只读计划，待用户确认）

## 当前基线（2026-10-19）

- 代码仓当前提交：`f4a97d2799135bc92d501a6b28c725bc933f7596`；当前分支：`task/P0-04-doctor`。
- 设定集历史触及两个提交：`67d788c640f4592ab7a372a1037424b2f336bfb6`（P2-01 建立）与 `ae803df8e70b4cfb759a910e1fc8c96c8ba28197`（P2-01 补充）。
- 下面列出当前已跟踪 Bible 文件的路径、字节数和工作树 SHA-256，作为迁移前基线。迁移尚未执行。

| 路径 | 字节数 | SHA-256 |
| --- | ---: | --- |
| `content/nemu/bible/00-original.png` | 2026786 | 5D53899BC0AABCE078D4A6E9570ECAB3CC0F9C0C9D2A877D84F242F6843C6A30 |
| `content/nemu/bible/01-turnaround-front-side-back.png` | 1555389 | 5D5DB049E3406E913AF08F9193A549EF8BB83A8F0C6D7326CA636E5DEF0D1FFD |
| `content/nemu/bible/02-expressions(1-4).png` | 1825536 | E6E43D567B9B63F799FBC2555EF5722137BCFCFD7DB7131C2EA278161601FD77 |
| `content/nemu/bible/02-expressions(5-8).png` | 1899973 | 00354C1E3EC7D8283C569687AA35672FE163BEEAF93AD393BDA0DEA58E750DE8 |
| `content/nemu/bible/02-expressions(9-12).png` | 1936071 | 9AFD9443C239976D516DB8265EADAC8A6836155F2B522FF5A7D0F453BA087DFC |
| `content/nemu/bible/03-palette.json` | 849 | 8F3C0910BDE588A3D27839DBC39A90F31E3E7EB90A5510FDD6E08A3C31B3F78F |
| `content/nemu/bible/04-proportions.md` | 2920 | E790B8339F961BA15668E26B93D9CC15D65F34771DFD51D41644BD627FED03E0 |
| `content/nemu/bible/05-outfit.md` | 3069 | 12AFAE92AD620B4EC10C8D5C69F485E2F6A73B4C642BAD4A7DC2492C49F65128 |
| `content/nemu/bible/06-persona-seed.md` | 3212 | 568F15F14B201D4B7D3B1A395E00CAF9D9BB7CA49D6A65C1581FFE4212D6FEF0 |
| `content/nemu/bible/07-genesis-notes.md` | 6087 | A4F4DE9BAC4E41FCCCB95256D67CC657AB9469CF1C2776F6F0F47C21DF0F9E73 |
| `content/nemu/bible/index.md` | 12647 | 360A3B63C0C86ADCA2E5758AC4C068466D61C77FF0DBAFA902DDD79DC511DB26 |


## 推荐迁移步骤（本文件不授权执行）

1. 用户确认迁移范围及 `nemu-content` 私有仓库地址；确认后先暂停对 `content/` 的并行编辑。
2. 创建只用于迁移的镜像副本或 Git bundle，记录原始 `HEAD`、`git status --short`、Bible 跟踪路径清单、每个文件 SHA-256，并保存仓库对象库备份。原始 `nemu` 仓库和工作区保持不动。
3. 在镜像副本中用 `git filter-repo` 仅保留 `content/nemu/bible/`，并重命名为内容仓库内的 `nemu/bible/`。这会保留 Bible 相关提交的作者、时间、说明和先后历史；过滤后的 commit id 会因树路径变化而重写。不要在主工作区做历史重写。
4. 对过滤仓库核对 `git log -- nemu/bible`、文件数量、相对路径、文件尺寸、每项 SHA-256；抽查原始两个 P2-01 提交及图片 blob。确认二进制 SHA-256 完全一致后，再向用户确认过的私有 `nemu-content` 仓库推送历史。
5. 为将来新增的内容资产配置独立 `.gitattributes`/LFS 规则；先在内容仓库验证 checkout 后文件 SHA-256，再决定是否启用 LFS，避免迁移时把 LFS pointer 当作源文件。
6. 在新分支中将代码仓跟踪的 `content/` 路径替换为指向 `nemu-content` 的子模块，保留原 Bible 文件在子模块路径 `content/nemu/bible/`；提交子模块 gitlink 与 `.gitmodules`。确认主仓库提交只改变子模块指针，不带入任何内容二进制。
7. 迁移后的工作树再次核对 11 个文件 SHA-256、历史日志、子模块 URL/私有访问和 `git submodule status`；再跑 P0-05 第二部分检查。

## 用户数据保护与回滚

- `content/nemu/source/` 目前是未跟踪的用户数据，含原始 VRM，SHA-256 为 `5484C94F54F6C000767FEBACBC99FD7EA23AC074D9CA8EECEBAF65B967DE39AB`。本次未复制、移动或写入它。获批实施前必须先单独盘点其全部路径与 SHA-256；迁移不能把它加入代码仓或内容仓提交。
- 如果内容仓子模块覆盖工作树前仍有未跟踪数据，先将这些数据做独立字节级备份并验证哈希；只有在确认恢复路径和哈希后才建立子模块工作树。迁移结束将用户文件恢复到预期的 `content/nemu/source/`，保持未跟踪状态，除非用户另行授权纳入内容仓。
- 回滚时，在子模块提交合并前可恢复代码仓原 `content/` tree 和原始 Bible commits；合并后若需回退，先撤销主仓库子模块 gitlink 提交，再从过滤仓库恢复内容文件。所有回滚操作都必须先验证原始 Bible SHA-256 与用户 `source/` 哈希。

## 待确认

- [ ] （阻塞）批准迁移方案与执行时机。
- [ ] （阻塞）提供/确认私有 `nemu-content` 仓库地址及其访问权限。
- [ ] （不阻塞）确定未来内容仓 LFS 规则；在确认前保持字节无损，不做 LFS 转换。