# P5-38 证据索引

## #240 合并后复验

源码为 `8d439f66b19ba8ad83671854d8dc371ca11e2d3a`。Windows 默认并行工作区、单独运行的 gateway、W19 组合测试、单轮耗时、fmt、Clippy 的退出码见 checks.tsv。表中长度与摘要按仓库 blob 计算；TSV 在入库时规范为 LF。review-check.log 保留开发中 SQLite 整数转换的编译错误，review-scale.log 保留长历史夹具未绑定前态修订号的首次失败，review-scale2.log 为扩大计时范围之前的开发运行。最终单轮数据见 round.log。

| 文件 | 字节 | SHA-256 |
| --- | ---: | --- |
| [checks.tsv](https://github.com/nanake-lab/nemu-evidence/blob/46f82d761444a46cb14b88a48e535700e801c08d/P5-38/checks/8d439f66/checks.tsv) | 51 | c6259e594145c548ddf515a752485aaf8dbdc7c463b8ee7e7b5996cf807adacc |
| [clippy.log](https://github.com/nanake-lab/nemu-evidence/blob/46f82d761444a46cb14b88a48e535700e801c08d/P5-38/checks/8d439f66/clippy.log) | 934 | 2d5ab930d260b0008bc2029b1cab2d6cfcdab576b507d6beae19ff67597166c5 |
| [fmt.log](https://github.com/nanake-lab/nemu-evidence/blob/46f82d761444a46cb14b88a48e535700e801c08d/P5-38/checks/8d439f66/fmt.log) | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| [gateway.log](https://github.com/nanake-lab/nemu-evidence/blob/46f82d761444a46cb14b88a48e535700e801c08d/P5-38/checks/8d439f66/gateway.log) | 5500 | 232953bcc7e1af43f4457bdd7f3dfc60416828a8293897c47e78240555fe76b9 |
| [review-check.log](https://github.com/nanake-lab/nemu-evidence/blob/46f82d761444a46cb14b88a48e535700e801c08d/P5-38/checks/8d439f66/review-check.log) | 3465 | faeb533e580c8ba93fa51f115fe5250c034ee362643f4c0b9c576a6954ffc8a8 |
| [review-scale.log](https://github.com/nanake-lab/nemu-evidence/blob/46f82d761444a46cb14b88a48e535700e801c08d/P5-38/checks/8d439f66/review-scale.log) | 2343 | 147cc966dd4f4b80cfa8cfdbfd086d71b6f7f245ec2f517c53f6fd3b853ce498 |
| [review-scale2.log](https://github.com/nanake-lab/nemu-evidence/blob/46f82d761444a46cb14b88a48e535700e801c08d/P5-38/checks/8d439f66/review-scale2.log) | 660 | 036e9e778811cc6c4695111cabe242ece7ca36fb8116b585c5a60f67438d92c5 |
| [round.log](https://github.com/nanake-lab/nemu-evidence/blob/46f82d761444a46cb14b88a48e535700e801c08d/P5-38/checks/8d439f66/round.log) | 739 | 5cb7bdf830cd9d1f4f77bcd5874ad92538689bf147e92373e4c41f7190e0a902 |
| [w19.log](https://github.com/nanake-lab/nemu-evidence/blob/46f82d761444a46cb14b88a48e535700e801c08d/P5-38/checks/8d439f66/w19.log) | 257 | b398df2e5653339df895eaac92dfa4a830dd62eab971d23cc99be81d9283f2e7 |
| [workspace.log](https://github.com/nanake-lab/nemu-evidence/blob/46f82d761444a46cb14b88a48e535700e801c08d/P5-38/checks/8d439f66/workspace.log) | 62225 | 5f9efe25254dac02233ff1a69e92874a04b3859068efcec770e2a797c5435065 |

## P1 复验

被测代码为 `49d45a90fd440a9d94ff9f93e352ce09006f0069`。原始日志与私有证据仓提交 `25164b3e20686e7ecab487343bf33a1f883aad18` 的 Git blob 已逐文件核对。

| 文件 | 字节 | SHA-256 |
| --- | ---: | --- |
| [P5-38/checks/store-runtime-49d45a90.log](https://github.com/nanake-lab/nemu-evidence/blob/25164b3e20686e7ecab487343bf33a1f883aad18/P5-38/checks/store-runtime-49d45a90.log) | 13262 | 171fd0431f287f6edfd48a51c9bc224d21f9bb79047ca5c0aef361a07af40096 |
| [P5-38/V2-88/w19-49d45a90.log](https://github.com/nanake-lab/nemu-evidence/blob/25164b3e20686e7ecab487343bf33a1f883aad18/P5-38/V2-88/w19-49d45a90.log) | 1624 | e06f0c18fc90c82a28adc0dce54b8ef8bcace391ee78276f52961e0b9291ad21 |
| [P5-38/checks/clippy-49d45a90.log](https://github.com/nanake-lab/nemu-evidence/blob/25164b3e20686e7ecab487343bf33a1f883aad18/P5-38/checks/clippy-49d45a90.log) | 2380 | 5acb79d597c841d18020f9bf632edb32bce8e8d9139175edce6165894bda25c5 |

## 首轮验收

以下文件保存在私有证据仓库 nanake-lab/nemu-evidence 的提交 4eadc5df815e42b38effb3e5917d9f54fefbc6b3。表内 SHA-256 按原始文件字节核算，并逐个与该提交的 Git blob 比对。development 收录开发阶段失败输出，workspace-superseded 在旧库测试失败后中止。checks/workspace-fa5155ab 是该源码的完整工作区运行。

| 文件 | 字节 | SHA-256 |
| --- | ---: | --- |
| [P5-38/SYS01/gateway-19845545-first.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/SYS01/gateway-19845545-first.log) | 5438 | de58e576bc44023fbf75fafc56222618d32153a57672954698b5e7d03ed0eedc |
| [P5-38/SYS01/gateway-19845545-second.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/SYS01/gateway-19845545-second.log) | 4982 | 3657866b2bbfdaf6079b7621e4e35d74acb456e61a1ca1f3366aab22f198283b |
| [P5-38/SYS01/gateway-469262e4-first.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/SYS01/gateway-469262e4-first.log) | 6845 | ec0fa7ee6bfe550af365959c4b26b6bc02dc5fa61d1feb2c5f6cba695dffda07 |
| [P5-38/SYS01/gateway-469262e4-second.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/SYS01/gateway-469262e4-second.log) | 5465 | cf6292824737386d08192a335dd9c7640f6e3a922ad68244998b416e0257d49e |
| [P5-38/SYS01/gateway-serial-19845545.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/SYS01/gateway-serial-19845545.log) | 4628 | 8b5d1955b8257e448b5263c30a0104b23542350e3f01dcf79c1c9d46cfd25247 |
| [P5-38/SYS17/format-three-19845545.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/SYS17/format-three-19845545.log) | 605 | c31687d7d5a5030f1ce7d05bfeb88dc7c5b80977b5c9d5033ee2a5acd9bb9438 |
| [P5-38/SYS17/format-three.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/SYS17/format-three.log) | 532 | 1e2ffd271084e9a33b8e9d11964787311e8d3d9d63ca921a4a0e3ed82df7e819 |
| [P5-38/SYS17/store-initial.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/SYS17/store-initial.log) | 8141 | 75ead330a9e3b4c8fdc8c43b48af02c938375789ceead473e20f7ad1372c5dad |
| [P5-38/V2-78/cognition-19845545.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/V2-78/cognition-19845545.log) | 1715 | f5115b2e91f5fd1c78db1b6fab2ca9159b73dc4c98feec361061d4cd086584ba |
| [P5-38/V2-78/rounds-extended.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/V2-78/rounds-extended.log) | 1367 | e20696fe456ed85708fdd6850eae4514fd204897c0f5223f9626fd0fac99e6fa |
| [P5-38/V2-78/rounds-fixed.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/V2-78/rounds-fixed.log) | 1185 | 95d47eaa2dc97b841a8ca05ce85ab2c8e861688969953945581c181772cf8e9b |
| [P5-38/V2-88/w19-19845545.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/V2-88/w19-19845545.log) | 717 | 3f35d43c1d1e3f14b4b8a0e8733497c09922e9d48bc7ed3909a95f6cc77dd6e8 |
| [P5-38/V2-88/w19.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/V2-88/w19.log) | 717 | 48710da8c77bde2de0d3a6b3b2edd3bf7c97d2a901becd99dfe4b5be577817f5 |
| [P5-38/checks/clippy-19845545.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/checks/clippy-19845545.log) | 533 | e0b79de0c4670d7894dc5093bc4aff48e020881d521978488223a77151b73f87 |
| [P5-38/checks/clippy.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/checks/clippy.log) | 1225 | dd01090cb971704440d4f41290bc9b06aa74eaa1aa1dd716ea3b40725744ad4d |
| [P5-38/checks/code-19845545.json](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/checks/code-19845545.json) | 1022 | e764a4ca05fdbd65066d3f4bc7679e9aeaf02b78a5786a9e0885b151830087c4 |
| [P5-38/checks/format-19845545.json](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/checks/format-19845545.json) | 180 | 824950648b7d567fb1d078cacbae9464bfb080abf3ffa6c7607212ac2f0cc254 |
| [P5-38/checks/format-fa5155ab.json](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/checks/format-fa5155ab.json) | 231 | 9fe97280f6a78655349ae8e9588dd768c4960952b77ff8e5e434830f97ba6323 |
| [P5-38/checks/workspace-fa5155ab.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/checks/workspace-fa5155ab.log) | 60503 | edb1d61eaf6a11ee96bbbf97b12c55bd7d2d0dc764b1e7a3113ac1f72334c3a0 |
| [P5-38/contracts/contracts.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/contracts/contracts.log) | 5293 | ae83252b320a4ca66f9f0d9e99596aeea5e5029c1a5aae31182ee02b832c5a13 |
| [P5-38/contracts/source-check.json](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/contracts/source-check.json) | 489 | 25f9370be892bd95dd8c144b5cc5dbbe8dc58f88293a8d000cc2c7dd34cec682 |
| [P5-38/development/check-initial.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/development/check-initial.log) | 2789 | 464e51717f5d95b04ac749656830b1fb4f9aed8e301d6f18efb51c4f730bab85 |
| [P5-38/development/rounds-first.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/development/rounds-first.log) | 854 | 1a55bd3585b453068503368363156e72e3d7930570e41ab7eaa58ff52e1b1b9a |
| [P5-38/development/rounds-second.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/development/rounds-second.log) | 1598 | e0fe0af8b6f32fb4fffa493e62550208259b48b3b97de1f9314ba5e886c9edc0 |
| [P5-38/development/workspace-superseded.log](https://github.com/nanake-lab/nemu-evidence/blob/4eadc5df815e42b38effb3e5917d9f54fefbc6b3/P5-38/development/workspace-superseded.log) | 49233 | e45a28ed85cdd6dca1eff3b11b47fda0b2c1d6b246680622fd82bef83d59a1e4 |
