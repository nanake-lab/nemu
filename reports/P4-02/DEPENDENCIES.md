# P4-02 实验环境依赖与许可登记

本次无新增依赖。以下是已固定实验环境的元数据登记，不等于正式产品分发审核；代码与预训练权重/随包数据分开。完整许可证原文、固定下载来源及哈希见 [证据索引](EVIDENCE-INDEX.md)，P7 条件见 [待决表](gsv-v5-resume-09/P7-PENDING.md)。

Distance 有意排除：g2p_en 2.1.0 元数据声明它，但 g2p.py / expand.py 未导入；因 AGENTS 3.8 的 GPL 禁令排除。g2p_en 采用不带依赖安装，显式补 nltk/inflect/numpy；后续禁网中文与中英混合真实 CPU 推理通过。不声称 g2p_en 的元数据依赖全部满足。

frozendict 2.5.1（LGPL-3.0-only）按用户批准：独立包动态导入、不修改、保留许可证、可替换、P7 复审。必经导入链为 models.py → f5_tts.model → DiT → x_transformers.x_transformers → einx → einx._src.frontend.ops → api → einx._src.util.lru_cache → frozendict。

FFmpeg n8.1.3-29-gad3b795b72-20261101 使用 BtbN win64-lgpl-shared-8.1 固定 autobuild-2026-11-01-12-49 发布，ZIP SHA-256 `786b43f77ea260af71fbb1bcdaabd43df83492122fb5575027f659d490d30b37`。配置含 --enable-version3、--enable-shared，不含 --enable-gpl / --enable-nonfree，随包许可证为 LGPL v3。只在进程内 add_dll_directory，独立未修改组件；许可/配置原文在私有 gsv-v5-resume-04/。speed_factor=1.0，固定上游 speed_change 调用已注释，不依赖系统 ffmpeg.exe。

| 代码包 | 版本 | 发行元数据中的许可 |
| --- | --- | --- |
| accelerate | 1.15.0 | License :: OSI Approved :: Apache Software License |
| aiohappyeyeballs | 2.7.1 | License :: OSI Approved :: Python Software Foundation License |
| aiohttp | 3.14.5 | Apache-2.0 AND MIT |
| aiosignal | 1.4.0 | License :: OSI Approved :: Apache Software License |
| annotated-types | 0.8.0 | MIT |
| anyio | 4.16.0 | MIT |
| attrs | 26.1.0 | MIT |
| audioread | 3.1.0 | MIT |
| budoux | 0.10.2 | Apache-2.0 |
| certifi | 2026.10.23 | License :: OSI Approved :: Mozilla Public License 2.0 (MPL 2.0) |
| cffi | 2.1.1 | MIT-0 |
| charset-normalizer | 3.5.2 | MIT |
| click | 8.5.0 | BSD-3-Clause |
| cloudpickle | 3.1.2 | License :: OSI Approved :: BSD License |
| cn2an | 0.5.24 | License :: OSI Approved :: MIT License |
| colorama | 0.4.6 | License :: OSI Approved :: BSD License |
| coloredlogs | 15.0.1 | License :: OSI Approved :: MIT License |
| colorlog | 6.12.0 | License :: OSI Approved :: MIT License |
| contourpy | 1.3.3 | License :: OSI Approved :: BSD License |
| cycler | 0.12.1 | License :: OSI Approved :: BSD License |
| decorator | 5.3.1 | BSD-2-Clause |
| defusedxml | 0.7.1 | License :: OSI Approved :: Python Software Foundation License |
| einops | 0.8.2 | License :: OSI Approved :: MIT License |
| einx | 0.4.3 | MIT |
| fast-langdetect | 0.3.2 | MIT |
| fastapi | 0.115.12 | License :: OSI Approved :: MIT License |
| fasttext-predict | 0.9.2.4 | License :: OSI Approved :: MIT License |
| ffmpeg-python | 0.2.0 | License :: OSI Approved :: Apache Software License |
| filelock | 4.2.4 | MIT |
| flatbuffers | 25.12.19 | License :: OSI Approved :: Apache Software License |
| fonttools | 4.61.1 | MIT |
| frozendict | 2.5.1 | LGPL-3.0-only |
| frozenlist | 1.8.0 | Apache-2.0 |
| fsspec | 2026.9.0 | BSD-3-Clause |
| future | 1.0.0 | License :: OSI Approved; License :: OSI Approved :: MIT License |
| g2p-en | 2.1.0 | Apache Software License |
| h11 | 0.16.0 | License :: OSI Approved :: MIT License |
| httpcore | 1.0.9 | BSD-3-Clause |
| httpx | 0.28.1 | License :: OSI Approved :: BSD License |
| huggingface_hub | 0.36.2 | License :: OSI Approved :: Apache Software License |
| humanfriendly | 10.0 | License :: OSI Approved :: MIT License |
| idna | 3.20 | BSD-3-Clause |
| inflect | 7.6.0 | MIT |
| jaraco.functools | 4.6.0 | MIT |
| jieba | 0.42.1 | License :: OSI Approved :: MIT License |
| jieba_fast | 0.53 | License :: OSI Approved :: MIT License |
| Jinja2 | 3.1.6 | License :: OSI Approved :: BSD License |
| joblib | 1.6.0 | BSD-3-Clause |
| jsonschema | 4.23.0 | License :: OSI Approved :: MIT License |
| jsonschema-specifications | 2025.9.1 | MIT |
| kiwisolver | 1.4.9 | License :: OSI Approved :: BSD License |
| lazy-loader | 0.6 | BSD-3-Clause |
| librosa | 0.10.2 | License :: OSI Approved :: ISC License (ISCL) |
| lightning-utilities | 0.15.3 | Apache-2.0 |
| llvmlite | 0.50.0 | BSD-2-Clause AND Apache-2.0 WITH LLVM-exception |
| loguru | 0.7.3 | License :: OSI Approved :: MIT License |
| MarkupSafe | 3.0.4 | BSD-3-Clause |
| matplotlib | 3.10.8 | License :: OSI Approved :: Python Software Foundation License |
| more-itertools | 11.2.1 | MIT |
| mpmath | 1.3.0 | License :: OSI Approved :: BSD License |
| msgpack | 1.2.4 | Apache-2.0 |
| multidict | 7.2.1 | Apache-2.0 |
| narwhals | 2.28.0 | MIT |
| networkx | 3.7 | BSD-3-Clause |
| nltk | 3.10.4 | License :: OSI Approved :: Apache Software License |
| numba | 0.68.1 | License :: OSI Approved :: BSD License |
| numpy | 1.26.4 | License :: OSI Approved :: BSD License |
| onnxruntime-gpu | 1.19.2 | License :: OSI Approved :: MIT License |
| OpenCC | 1.4.2 | License :: OSI Approved :: Apache Software License |
| packaging | 26.3 | Apache-2.0 OR BSD-2-Clause |
| peft | 0.17.1 | License :: OSI Approved :: Apache Software License |
| pillow | 12.0.0 | MIT-CMU |
| platformdirs | 4.14.3 | MIT |
| pooch | 1.9.0 | BSD-3-Clause |
| proces | 0.1.7 | License :: OSI Approved :: MIT License |
| propcache | 0.5.4 | Apache-2.0 |
| protobuf | 7.36.2 | 3-Clause BSD License |
| psutil | 7.2.2 | BSD-3-Clause |
| pycparser | 3.12 | BSD-3-Clause |
| pydantic | 2.14.2 | MIT |
| pydantic_core | 2.50.2 | MIT |
| pyparsing | 3.2.5 | MIT |
| pypinyin | 0.55.0 | License :: OSI Approved :: MIT License |
| pyreadline3 | 3.5.6 | License :: OSI Approved :: BSD License |
| python-dateutil | 2.9.0.post0 | License :: OSI Approved :: BSD License; License :: OSI Approved :: Apache Software License |
| pytorch-lightning | 2.6.6 | License :: OSI Approved :: Apache Software License |
| PyYAML | 6.0.3 | License :: OSI Approved :: MIT License |
| referencing | 0.37.0 | MIT |
| regex | 2026.9.29 | Apache-2.0 AND CNRI-Python |
| requests | 2.34.2 | License :: OSI Approved :: Apache Software License |
| resampy | 0.4.3 | License :: OSI Approved :: ISC License (ISCL) |
| robust-downloader | 0.0.2 | License :: OSI Approved :: Apache Software License |
| rotary-embedding-torch | 0.9.1 | License :: OSI Approved :: MIT License |
| rpds-py | 2026.9.1 | MIT |
| safetensors | 0.8.0 | License :: OSI Approved :: Apache Software License |
| scikit-learn | 1.9.1 | BSD-3-Clause |
| scipy | 1.17.1 | License :: OSI Approved :: BSD License |
| setuptools | 84.2.1 | MIT |
| six | 1.17.0 | License :: OSI Approved :: MIT License |
| soundfile | 0.14.0 | License :: OSI Approved :: BSD License |
| soxr | 1.1.0 | LGPL-2.1-or-later |
| split-lang | 2.1.0 | License :: OSI Approved :: MIT License |
| starlette | 0.46.2 | BSD-3-Clause |
| structlog | 25.3.0 | MIT OR Apache-2.0 |
| sympy | 1.14.0 | License :: OSI Approved :: BSD License |
| threadpoolctl | 3.7.0 | BSD-3-Clause |
| tokenizers | 0.22.2 | License :: OSI Approved :: Apache Software License |
| torch | 2.10.0+cu128 | BSD-3-Clause |
| torchaudio | 2.10.0+cu128 | License :: OSI Approved :: BSD License |
| torchcodec | 0.10.0 | BSD 3-Clause License |
| torchmetrics | 1.5.2 | License :: OSI Approved :: Apache Software License |
| tqdm | 4.70.1 | MPL-2.0 AND MIT |
| transformers | 4.57.3 | License :: OSI Approved :: Apache Software License |
| typeguard | 4.6.0 | MIT |
| typing-inspection | 0.4.4 | MIT |
| typing_extensions | 4.16.0 | PSF-2.0 |
| urllib3 | 2.8.0 | MIT |
| uvicorn | 0.34.2 | BSD-3-Clause |
| websockets | 15.0.1 | License :: OSI Approved :: BSD License |
| win32_setctime | 1.2.1 | License :: OSI Approved :: MIT License |
| wordsegment | 1.3.1 | License :: OSI Approved :: Apache Software License |
| x-transformers | 1.44.8 | License :: OSI Approved :: MIT License |
| yarl | 1.25.1 | Apache-2.0 |

## 代码、预训练权重与随包数据

| 类别 | 名称 / 固定版本 | 许可及条件 |
| --- | --- | --- |
| 上游代码 | GPT-SoVITS 04a44eb977586645f33594b72af13a035a2056c6 | MIT |
| 预训练权重 | lj1995/GPT-SoVITS 8f8857563c040d542de108a2c18b0425abd6c9c5 | 官方模型卡 MIT；download-plan 和 verification 记录完整大小/SHA；不入代码仓 |
| 预训练权重 | G2PW / XXXXRT 固定 0c47645e02a7bc3688d7b263b0042c81e3cd82cd | 模型仓库卡 MIT；ZIP SHA-256 46292be0374a49308069233cd5c147ae4c41806558e4781a2467a31a4d8099da |
| 预训练权重 | fastText lid.176.bin（131266198 B） | CC-BY-SA-3.0；SHA-256 7e69ec5451bc261cc7844e49e4792a85d7f09c06789ec800fc4a44aec362764e；独立、不修改、署名、P7 复审 |
| 随包模型 | fast-langdetect 0.3.2 的 lid.176.ftz | CC-BY-SA-3.0；不以包代码 MIT 替代模型许可；同上条件 |
| 随包数据 | NLTK CMUdict 0.6 | 用户批准研究/商业不受限制的原文，保留 Carnegie Mellon University 出处 |
| 随包词频 | wordsegment 1.3.1 | 本机测试批准；来源 Google Web Trillion Word Corpus/Norvig，数据许可未明确，P7 前待决，仅影响英文罕见词拆分 |
| 派生词表 | 固定 GPT-SoVITS 内词典/拼音表 | 按用户批准以仓库 MIT 为本机依据，P7 复审 |

fastText 署名：Armand Joulin、Edouard Grave、Piotr Bojanowski、Tomas Mikolov；模型 [CC-BY-SA-3.0](https://creativecommons.org/licenses/by-sa/3.0/)。Pillow 12.0.0 按用户批准登记 MIT-CMU；matplotlib 3.10.8 及新增传递依赖见上表。soxr/soundfile 的既有 LGPL 动态加载、不修改和 P7 分发复核条件继续有效。

G2PW 在固定上游看到 CUDA provider 即自动用 GPU，不受 TTS device 参数控制；正式 provider 必须显式选择。当前实验只在进程内限定 CPUExecutionProvider；ORT 1.19.2 没有 preload_dlls，上游跳过，升级时重核。
