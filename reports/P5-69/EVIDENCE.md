# P5-69 证据索引

私有仓库：`nanake-lab/nemu-evidence`。证据提交：`d98c72a136b8156ef76e72342a4818d4b500524b`。
最终代码：`b8fe03ac9760aaa2f78bfced43ea8a2d2dc1b2b1`；基线：`51298e9ce5d571e8753d047eaac864fce007b867`。

逐文件核对 Git blob 与本机原始字节完全相同后计算 SHA-256；归档关闭自动行尾转换。日志仅含合成输入；没有读取正式或排练存储。

`d07-final.log` 包含 AC2-05/08/14/18/19 的纯 C 检查及 D07-A/E07 机制轨迹；生产接线和真实时间不在证据内。`kernel-final.log` 是最终完整测试；`kernel-02.log` 仅中间版。`initial-kernel.log` 和首次 CI 为已修复失败，保留原始结果。

| 私有证据路径 | SHA-256 |
| --- | --- |
| [P5-69/.gitattributes](https://github.com/nanake-lab/nemu-evidence/blob/d98c72a136b8156ef76e72342a4818d4b500524b/P5-69/.gitattributes) | `705fd4d6451a31d36b3df7de96f83f30ac976c9b4a6d1e51671d8e2f33e2d0da` |
| [P5-69/validation-20261110/ci-42838938760-failed.log](https://github.com/nanake-lab/nemu-evidence/blob/d98c72a136b8156ef76e72342a4818d4b500524b/P5-69/validation-20261110/ci-42838938760-failed.log) | `c70ba09ff9d69265cca7c19ed138bcc4be4050600ac23ebede668086916f771b` |
| [P5-69/validation-20261110/ci-42839438093-passed.log](https://github.com/nanake-lab/nemu-evidence/blob/d98c72a136b8156ef76e72342a4818d4b500524b/P5-69/validation-20261110/ci-42839438093-passed.log) | `fd14e31f600cf5dc7ba70237da3732627212c09188e428da643200498d66b8a5` |
| [P5-69/validation-20261110/ci-42841073030-passed.log](https://github.com/nanake-lab/nemu-evidence/blob/d98c72a136b8156ef76e72342a4818d4b500524b/P5-69/validation-20261110/ci-42841073030-passed.log) | `1371d51717de889a93927a6a3563b474e604be45472db171ebd3fb1c2f48e75c` |
| [P5-69/validation-20261110/clippy-final.log](https://github.com/nanake-lab/nemu-evidence/blob/d98c72a136b8156ef76e72342a4818d4b500524b/P5-69/validation-20261110/clippy-final.log) | `71ae38517ae215f9485e4d6f74e72d35fa2ab6e41df4be77958614e95c3044df` |
| [P5-69/validation-20261110/d07-final.log](https://github.com/nanake-lab/nemu-evidence/blob/d98c72a136b8156ef76e72342a4818d4b500524b/P5-69/validation-20261110/d07-final.log) | `c1395c1227dd3bed836770f08d16901d41128b35357ea1e69384d687706892e2` |
| [P5-69/validation-20261110/fmt-final.log](https://github.com/nanake-lab/nemu-evidence/blob/d98c72a136b8156ef76e72342a4818d4b500524b/P5-69/validation-20261110/fmt-final.log) | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| [P5-69/validation-20261110/initial-kernel.log](https://github.com/nanake-lab/nemu-evidence/blob/d98c72a136b8156ef76e72342a4818d4b500524b/P5-69/validation-20261110/initial-kernel.log) | `1adbcfea533b8e9cb2ecdf63a98c489a7b6b64b9920a2830131ae5aeab489aac` |
| [P5-69/validation-20261110/kernel-02.log](https://github.com/nanake-lab/nemu-evidence/blob/d98c72a136b8156ef76e72342a4818d4b500524b/P5-69/validation-20261110/kernel-02.log) | `f03d43419738eca9947be2c33a87a76b448343dc060fac80d78277af58f2d701` |
| [P5-69/validation-20261110/kernel-final.log](https://github.com/nanake-lab/nemu-evidence/blob/d98c72a136b8156ef76e72342a4818d4b500524b/P5-69/validation-20261110/kernel-final.log) | `f9f8ec8e346d2d3af06214af2d6b6dc10bb55828afd8956420121cd333638101` |
| [P5-69/validation-20261110/mypy-01.log](https://github.com/nanake-lab/nemu-evidence/blob/d98c72a136b8156ef76e72342a4818d4b500524b/P5-69/validation-20261110/mypy-01.log) | `8e63ae22e8213e327a5e9cbd584067681681bd8ef7f25c869877887ccdcfc4ff` |
| [P5-69/validation-20261110/mypy-final-02.log](https://github.com/nanake-lab/nemu-evidence/blob/d98c72a136b8156ef76e72342a4818d4b500524b/P5-69/validation-20261110/mypy-final-02.log) | `8e63ae22e8213e327a5e9cbd584067681681bd8ef7f25c869877887ccdcfc4ff` |
| [P5-69/validation-20261110/mypy-final.log](https://github.com/nanake-lab/nemu-evidence/blob/d98c72a136b8156ef76e72342a4818d4b500524b/P5-69/validation-20261110/mypy-final.log) | `8e63ae22e8213e327a5e9cbd584067681681bd8ef7f25c869877887ccdcfc4ff` |
| [P5-69/validation-20261110/oracle-final.log](https://github.com/nanake-lab/nemu-evidence/blob/d98c72a136b8156ef76e72342a4818d4b500524b/P5-69/validation-20261110/oracle-final.log) | `b5a6eeab73857670a180516dc625ed30127f2404cf7bb89c4d78e6999ccdbf92` |
| [P5-69/validation-20261110/ruff-01.log](https://github.com/nanake-lab/nemu-evidence/blob/d98c72a136b8156ef76e72342a4818d4b500524b/P5-69/validation-20261110/ruff-01.log) | `a4443afdcfb6d7363adb285762515ccf7cf50473b1a05c20c1a50f6bed4d26b0` |
| [P5-69/validation-20261110/ruff-final-02.log](https://github.com/nanake-lab/nemu-evidence/blob/d98c72a136b8156ef76e72342a4818d4b500524b/P5-69/validation-20261110/ruff-final-02.log) | `a4443afdcfb6d7363adb285762515ccf7cf50473b1a05c20c1a50f6bed4d26b0` |
| [P5-69/validation-20261110/ruff-final.log](https://github.com/nanake-lab/nemu-evidence/blob/d98c72a136b8156ef76e72342a4818d4b500524b/P5-69/validation-20261110/ruff-final.log) | `a4443afdcfb6d7363adb285762515ccf7cf50473b1a05c20c1a50f6bed4d26b0` |
| [P5-69/validation-20261110/validation.json](https://github.com/nanake-lab/nemu-evidence/blob/d98c72a136b8156ef76e72342a4818d4b500524b/P5-69/validation-20261110/validation.json) | `0b80fda1ecd3b6b6bddcf8752b509a8cd98e1d4d8895b281c3d3d4dac4ecad62` |
