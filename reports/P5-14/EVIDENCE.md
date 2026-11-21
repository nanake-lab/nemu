# P5-14 原始证据索引

行动专项的代码提交为 `6d2d3ecd32b8d5584b3b68ac22c2d7e330cf8d68`。工具链为 Windows x86_64 MSVC、Rust/Cargo 1.99.0。专项目录在 `core/` 下运行以下命令，16 项通过，一项依用户决定忽略。输出的一个 filtered out 是 core-runtime 其他模块的用例。

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --release -p core-runtime --lib action::tests --locked -- --nocapture --test-threads=1
cargo test --release --workspace --locked --no-fail-fast
```

行动专项中，25 ms 合成审批窗口在 25.6333 ms 时判定超时，从显示到观察到取消持久化为 2344 ms。X04 含创建临时谱系、子进程、重开与迟到结算，全程 19332 ms。这些是测试用时。

以下文件位于私有证据仓库的 `P5-14/`。文件名链接固定到包含该文件的提交，SHA-256 校验下载的 gzip 文件，解压后得到原始字节。

| 文件 | SHA-256 | 对应运行 |
| --- | --- | --- |
| [action-6d2d.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/d3d3b40198f00112e009767ac58c4c93c23c0dfa/P5-14/action-6d2d.log.gz) | `d2678b1ff5bc7c8008b5c8c985a3a42a3e86bc663ca9ea6dc28b197bc460cc8c` | 本次行动专项，16 通过，1 not_run |
| [action-a3bc.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/d3d3b40198f00112e009767ac58c4c93c23c0dfa/P5-14/action-a3bc.log.gz) | `08ec1d3c4fa61c9a82aa4719cfea7285c7833d124340f706dac8fca83c0b8016` | a3bc5a58 行动专项 |
| [action-final.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/d3d3b40198f00112e009767ac58c4c93c23c0dfa/P5-14/action-final.log.gz) | `605af6b558d1bc57dc9c2401a29a12737870575197f3621da75275a1ea7d4e7c` | 278d006f 行动专项 |
| [action-tests-02.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/d3d3b40198f00112e009767ac58c4c93c23c0dfa/P5-14/action-tests-02.log.gz) | `b43e35a1df74ecff91842539fcea7016ca6da320a686d8b63a87f35aec66c0bb` | 开发阶段 5 个用例 |
| [action-tests-03.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/d3d3b40198f00112e009767ac58c4c93c23c0dfa/P5-14/action-tests-03.log.gz) | `4296964283acb0e60b1a7384e6dcfabd898d750cc3d5b86f45cd98e5f3c222fe` | 开发阶段 8 个通过、1 个暂缓 |
| [ci-fmt-4cd6.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/d3d3b40198f00112e009767ac58c4c93c23c0dfa/P5-14/ci-fmt-4cd6.log.gz) | `ff5177ea1892faf04f0b3a6ab604c5a54320216e6dbb6894604175b20c334e82` | 4cd6582d Ubuntu 格式检查失败 |
| [clippy-01.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/d3d3b40198f00112e009767ac58c4c93c23c0dfa/P5-14/clippy-01.log.gz) | `514b3d0d86799d8e59e3ee6e9249c101b0f5e0f2ac4ef966c48fc35aa725cc23` | 开发期间 Clippy 枚举大小差异 |
| [clippy-02.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/d3d3b40198f00112e009767ac58c4c93c23c0dfa/P5-14/clippy-02.log.gz) | `7dc63d97bb78dd2435720accf8d86f581acadd15ba54cb402c9fa4f7b6632441` | 修正枚举后 Clippy |
| [clippy-6d2d.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/d3d3b40198f00112e009767ac58c4c93c23c0dfa/P5-14/clippy-6d2d.log.gz) | `108c0da38e508502d026be525b0cbb223fe7bbab9f681e0c352ab5c5c77bb7bc` | 本次全工作区 Clippy |
| [clippy-a3bc.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/d3d3b40198f00112e009767ac58c4c93c23c0dfa/P5-14/clippy-a3bc.log.gz) | `c6bfc8c5721bcc5ff227a95cff538f5a2b5ee741adbf89916a71e253a41b95d2` | a3bc5a58 Clippy |
| [clippy-final.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/d3d3b40198f00112e009767ac58c4c93c23c0dfa/P5-14/clippy-final.log.gz) | `b40ec2cc4320ffb80cc9b124986435145bceff3f37645f7de4a63fa10419df7c` | 278d006f Clippy |
| [commands-6d2d.txt.gz](https://github.com/nanake-lab/nemu-evidence/blob/c53006cebbe9d707e889e78ec63ed3367d67ecb9/P5-14/commands-6d2d.txt.gz) | `e6999bb79a0371983b142f0107f869bcd6197c65b77b884504db8eb7d830e9ad` | 本次运行命令、工具版本与退出码 |
| [first-action-tests.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/d3d3b40198f00112e009767ac58c4c93c23c0dfa/P5-14/first-action-tests.log.gz) | `374ff8b04676092b547d6977ef7357734c8569c227155dae4a151bdd84bdbf1b` | 开发期间首次编译失败 |
| [fmt-final.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/d3d3b40198f00112e009767ac58c4c93c23c0dfa/P5-14/fmt-final.log.gz) | `46b50c321b39e89a491b6727a01628c34245605a30beb3e7414c5e01cff90e6e` | 278d006f 格式检查，通过时无输出 |
| [fmt.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/d3d3b40198f00112e009767ac58c4c93c23c0dfa/P5-14/fmt.log.gz) | `46b50c321b39e89a491b6727a01628c34245605a30beb3e7414c5e01cff90e6e` | 开发阶段格式检查，通过时无输出 |
| [workspace-6d2d.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/c53006cebbe9d707e889e78ec63ed3367d67ecb9/P5-14/workspace-6d2d.log.gz) | `9eb4b8e6b798e315cdc3d24d65e73746f93839f9433138261684110e1633f69b` | 本次 release 全工作区运行 |
| [workspace-final.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/d3d3b40198f00112e009767ac58c4c93c23c0dfa/P5-14/workspace-final.log.gz) | `b6ad58bfc8cb247375b03b4dcfcc140858efddcb2edce9f0cd8cebf8689eee9a` | 278d006f 全工作区检查，修改签发边界后中止 |
| [workspace-release.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/d3d3b40198f00112e009767ac58c4c93c23c0dfa/P5-14/workspace-release.log.gz) | `9fb6bdbb3f427fde915f6325452db6757c6122e60ed237afcfa022645301d040` | 9d8dbd3e 全工作区检查，来源版本修正后中止 |
| [action-6d79.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/2f20bcd060ff2ad658ab0e4fa9120d35a093227b/P5-14/action-6d79.log.gz) | `ddd181115c6728566084f0fbb8da6605e534d5d082cf530b1fbadaa4fa87507a` | P1 修复后，使用默认设置签发及派发的行动专项 |
| [clippy-6d79.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/2f20bcd060ff2ad658ab0e4fa9120d35a093227b/P5-14/clippy-6d79.log.gz) | `2dd3cbe6c8aaf68b9f187a8c318f94f87c0a71b09a5a08526a564051e270c749` | P1 修复后全工作区 Clippy |
| [windows-82593109.log.gz](https://github.com/nanake-lab/nemu-evidence/blob/2f20bcd060ff2ad658ab0e4fa9120d35a093227b/P5-14/windows-82593109.log.gz) | `f64a480afacc290cbc9dcce53c1e09885b9a29adf7616d59cb79c5e429e04d74` | 旧提交 Windows CI 的 W18/SYS01 失败 |
