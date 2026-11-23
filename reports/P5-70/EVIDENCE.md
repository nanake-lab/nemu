# P5-70 证据索引

原始日志位于私有仓库 [nanake-lab/nemu-evidence](https://github.com/nanake-lab/nemu-evidence)，分支 `evidence/P5-70`。

证据提交：`1a087f1a020263f6f9c26be98cda97428e51ec8c`。下表 SHA-256 基于文件原始字节，并逐文件与该提交的 Git blob 比较一致；本目录 `* -text` 禁止换行转换。全部文件小于 1 MB，无分片。

代码验收提交：`7a94bda3b527764f23d23e4de08e87adc73e2378`；基线 B 与最后检查的 main M 均为 `51298e9ce5d571e8753d047eaac864fce007b867`。fast 子树为 `3e715cccc679b2c981c2a9fe452584d83fe79004`，已核对远端相同。后续报告提交仅改变 reports/P5-70，不改变此代码树。

- `kernel-01`：初轮全包回归（257/0/1，706.98 秒），输入哈希单列；后续治理引用与角色边界补充不由此轮验收。
- `kernel-final`：最终代码全包回归（257/0/1，733.12 秒）；运行 JSON 含命令、起止时间、提交与退出码。
- `clippy-final`、`fmt-final`：退出码均 0；空 fmt 日志是无诊断输出，运行 JSON 保留命令和退出码。
- `oracle-final`：Python 3.12.10、种子 5070、229 向量、3125 距离比较；再生哈希相同，分解残差如报告。
- `ci-42839385288`：[Ubuntu CI](https://github.com/nanake-lab/nemu/actions/runs/42839385288)，成功；包含 workspace clippy、workspace tests（按现有 CI 排除 core-store）和 W19 CPU compile。日志是 GitHub API 返回的解码文本；jobs JSON 记录逐步骤状态，未执行步骤不计作通过。
- `environment` 与 `final-inputs`：工具链/临时目录/并发设置及 fast 源文件 SHA-256。
- 历史 ignored 与 LAW-6 失败诊断保留，含义见 REPORT.md；本 PR 没有新增跳过或放宽断言。

| 私有仓库路径 | 字节数 | SHA-256 |
| --- | ---: | --- |
| `P5-70/.gitattributes` | 8 | `705fd4d6451a31d36b3df7de96f83f30ac976c9b4a6d1e51671d8e2f33e2d0da` |
| `P5-70/ci-42839385288-jobs.json` | 4910 | `01ca506f08347458dba87b7187ad846225f21b40ecc15aba8cc3f704d58fd1ac` |
| `P5-70/ci-42839385288.log` | 123528 | `c65111055ae18e6d6306f16fd429cbbd28004f8ec933021c1aa7afaadd027fec` |
| `P5-70/clippy-final-run.json` | 300 | `50f2806e51b8c0afcc04ca7b93b67d0f74fdaa2e19527179a91e132cae568148` |
| `P5-70/clippy-final.log` | 722 | `eea32cb1d094c6a9e4e96e90c830758089efa3d462098b5fd692b37f9c5b5de0` |
| `P5-70/environment.json` | 596 | `0debe803d3625573c22379eafdebc90c21636511a7595b418bea5275b6682d9f` |
| `P5-70/final-inputs.json` | 3552 | `d771b8a0d5d991ae1a9edb9b545ca84dec538e063da54ac5c8bbd88eeec7dc86` |
| `P5-70/fmt-final-run.json` | 208 | `f3df265312d26b925e9f38e9085247fa92792bb8e3aa4bc30ffc30bb1d70b23a` |
| `P5-70/fmt-final.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `P5-70/kernel-01-inputs.json` | 3383 | `ade3630a1d4b1f44664ae41d275c4c153765fd6229c5f027ce279a0b7b87d453` |
| `P5-70/kernel-01.log` | 49891 | `9a16236bcad79ac440ffe4e656fb897957484c71d4841e0c418f1e28ec6cce4c` |
| `P5-70/kernel-final-run.json` | 269 | `6e3744e7494b539236a1cef03e8ffa7300c143cdda9529ace08c01c3e5993824` |
| `P5-70/kernel-final.log` | 25382 | `629e5bbd7220e827eb06aa369b2aa8ad882d6669f894d4da68592ef0481b8c59` |
| `P5-70/oracle-final.json` | 546 | `f573fe19b774dae59060e32be082a1b1147af07e89abd199070b01d629140768` |

## 2026-11-11 审查建议增量

私有证据提交：`b81383094901f4d37ad36ac0f863b3f417228bbc`，以下 16 个文件与 Git blob 逐字节核对一致；原版证据不改写。

首轮代码 `31510bcdebd82db74d2632c8e63f7fb6914b8f45`：kernel 257/0/1、697.08 秒；clippy 与 CI 42857935159 因两处 clone_on_copy 失败。最终代码 `723b14f5fa913d60026fbcef76da804ce9824bb6` 去掉多余 clone 后，kernel 257/0/1、336.19 秒，clippy / fmt 退出码 0。kernel-final 为单一测试命令、4 个测试线程；其他重负载命令串行。全部数据为合成夹具。

`inputs` 与 `final-inputs` 分别绑定两次代码；各 run JSON 记录命令、提交及退出码。`ci-failed.log` 为失败 job 129975955709 的 API 解码日志；`ci-observed.json` 是当时查询快照，不代表之后的 CI 状态。原有 ignored 没有改变。

| 私有仓库路径 | 字节数 | SHA-256 |
| --- | ---: | --- |
| `P5-70/review-followup/ci-failed.log` | 52885 | `ed4597de9030e12c0ac623839146e00b754b1ea9a511ae8ddac2ad7f7c28024f` |
| `P5-70/review-followup/ci-observed.json` | 479 | `f88e27034445b03959684eee61a293d38dbe8a485cb104a9f798a21cfbb9e7f5` |
| `P5-70/review-followup/clippy-final-run.json` | 257 | `67402bc3c0340c4ba554ac464fa7682bba28a5a8da2436215f7f2420a725f427` |
| `P5-70/review-followup/clippy-final.log` | 153 | `a7747e8c49d5cf67f8015c68a3a2ca065f8524728042ce5a1c74ca3ec5803aef` |
| `P5-70/review-followup/clippy-run.json` | 302 | `b086ea198f5aa9c07620d9a799b07b928ab9cf92e04af1d7689c09d96d4f7481` |
| `P5-70/review-followup/clippy.log` | 1557 | `2ad4976dd35319291ae5af56a1f97330fb122a0ed785ed904b223387031299f0` |
| `P5-70/review-followup/final-inputs.json` | 3731 | `31d8011af0e12b8defad54eca9c5cdaed6c619a271a243a51c9fcf0e5c1d6db2` |
| `P5-70/review-followup/fmt-final-run.json` | 155 | `110eff443368306893b31c7e2ff0f553ea4fe6ad0e52aa51f26fe14c73399517` |
| `P5-70/review-followup/fmt-final.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `P5-70/review-followup/fmt-run.json` | 208 | `ac4386d1137aaee77d557cffb660a31790d16a6e2a92f3569fd8d98554a22575` |
| `P5-70/review-followup/fmt.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `P5-70/review-followup/inputs.json` | 3973 | `fae710763407792f752aed49382b1d74ee08042aa5e73a07e095eae7c725f09b` |
| `P5-70/review-followup/kernel-final-run.json` | 311 | `eb8bd6d0d83d55dc8cc38c52ae78ffb419be709db9e1be12e86ab23c7eca0878` |
| `P5-70/review-followup/kernel-final.log` | 25787 | `e9919580d5da604b5acee91c381f919349b4617b049bd798f5795c51603a061f` |
| `P5-70/review-followup/kernel-run.json` | 311 | `62f542c0ae39e8e7a4282ab8c1e2722fc58e07ae98bb52df0f4110b115247522` |
| `P5-70/review-followup/kernel.log` | 26110 | `7dd086100cfd19fccd1e2709d22249121d39399005c4d5ed96ca0f5a2df195e6` |
