# FIX-STORE-01 证据索引

私有证据仓：`nanake-lab/nemu-evidence`；提交：`44f9fc116af0d356d729f1d9643dfbab710bf1d9`。

被测实现：`b5f5d58f03d5c7907e7fb00549e3ed542dc23f9a`；main 基线：`ce115abe6ced3ec8cc301bea2a290f47ace4807a`。本机缓存保留在 `D:/Dev/cache/FIX-STORE-01/`。

| 文件 | SHA-256 |
| --- | --- |
| [FIX-STORE-01/clippy-final.log](https://github.com/nanake-lab/nemu-evidence/blob/44f9fc116af0d356d729f1d9643dfbab710bf1d9/FIX-STORE-01/clippy-final.log) | `0cc9c74bde715f74517743fc7b516217bc77bad8d78d3d2a732d5e400a6f1dce` |
| [FIX-STORE-01/clippy.log](https://github.com/nanake-lab/nemu-evidence/blob/44f9fc116af0d356d729f1d9643dfbab710bf1d9/FIX-STORE-01/clippy.log) | `2d0667aa87492b92fce2d0f35b687f60e5468396f51d06196532ff2e0dd99833` |
| [FIX-STORE-01/consistency-first.log](https://github.com/nanake-lab/nemu-evidence/blob/44f9fc116af0d356d729f1d9643dfbab710bf1d9/FIX-STORE-01/consistency-first.log) | `ca5e736f03896eb9d74f712cfd0b075cbce23765496331c2a0d7b2236aa5dc63` |
| [FIX-STORE-01/consistency-second.log](https://github.com/nanake-lab/nemu-evidence/blob/44f9fc116af0d356d729f1d9643dfbab710bf1d9/FIX-STORE-01/consistency-second.log) | `08188fdfa37a2bbdabf9aa4fe28a4cb840806e38924afb5093e2cb2e13e8da40` |
| [FIX-STORE-01/final-test-head.txt](https://github.com/nanake-lab/nemu-evidence/blob/44f9fc116af0d356d729f1d9643dfbab710bf1d9/FIX-STORE-01/final-test-head.txt) | `1fba74b69132307141d43a894a05f92e4e95957010fc4303248b93053687998d` |
| [FIX-STORE-01/intermediate-stop.txt](https://github.com/nanake-lab/nemu-evidence/blob/44f9fc116af0d356d729f1d9643dfbab710bf1d9/FIX-STORE-01/intermediate-stop.txt) | `0773a8c539bdfc155da30bf5d10e4dacbfdbc47e2499f42dd0ef7f048f95fe93` |
| [FIX-STORE-01/numeric.log](https://github.com/nanake-lab/nemu-evidence/blob/44f9fc116af0d356d729f1d9643dfbab710bf1d9/FIX-STORE-01/numeric.log) | `baf32ca532462abb9fc114745ed88313cd424023182847101da276cffeefbdfc` |
| [FIX-STORE-01/validation.json](https://github.com/nanake-lab/nemu-evidence/blob/44f9fc116af0d356d729f1d9643dfbab710bf1d9/FIX-STORE-01/validation.json) | `9f637330c5576381dbd68e8142aa019ce48f5df5b7a2955abc9cb53b675cf6a4` |
| [FIX-STORE-01/workspace-final.log](https://github.com/nanake-lab/nemu-evidence/blob/44f9fc116af0d356d729f1d9643dfbab710bf1d9/FIX-STORE-01/workspace-final.log) | `e89e01861493c78e734539b5a9d235bf9c1539f37e008f78d18972314aa8dfac` |
| [FIX-STORE-01/workspace-tests.log](https://github.com/nanake-lab/nemu-evidence/blob/44f9fc116af0d356d729f1d9643dfbab710bf1d9/FIX-STORE-01/workspace-tests.log) | `d810efbcce77892be016cd3aef98489e1da8503976976821b9ac6830e36d96ce` |

最终验收为 `workspace-final.log`、`clippy-final.log` 与 `validation.json`。`consistency-first.log` 为已修复的首轮失败，`workspace-tests.log` 为主动中止的中间版本运行；均保留，不作为最终通过证据。格式检查无 stdout，退出码在 validation.json 中记录。
