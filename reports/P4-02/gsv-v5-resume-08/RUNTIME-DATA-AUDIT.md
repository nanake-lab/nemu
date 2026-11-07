# 运行时下载与随包数据集中核对（2026-11-03）

基线 GPT-SoVITS 04a44eb977586645f33594b72af13a035a2056c6；实际包版本见 resume-06/installed-dependencies.json。中文为 all_zh，中英混合为 zh，参考中文；不安装训练/UI/ASR。静态搜索覆盖 TTS_infer_pack、text、feature_extractor、module，以及 g2p_en、fast_langdetect、robust_downloader、split_lang、wordsegment、jieba/jieba_fast、pypinyin、cn2an、opencc；另检查 transformers/huggingface_hub、NLTK、librosa、torchaudio、torchmetrics、pytorch_lightning 下载接口。它不是对所有可能动态导入的数学证明，禁网运行负责发现漏项；当前尚未运行真实禁网合成。

| 文件/入口 | 来源 | 用途 | 数据许可（不替代为代码许可） | 中文/混合路径与准备状态 |
| --- | --- | --- | --- | --- |
| lid.176.bin；fast_langdetect ModelDownloader | https://dl.fbaipublicfiles.com/fasttext/supervised-models/lid.176.bin | 语言识别 | CC-BY-SA-3.0；用户 11-03 批准独立、不改、保留署名/许可、P7 复审 | 两者可触发；预置 pretrained_models/fast_langdetect；下载未完整，SHA 待完成 |
| fast_langdetect/resources/lid.176.ftz | fast-langdetect 0.3.2 wheel；fastText lid.176 | 包内低内存语言模型 | CC-BY-SA-3.0；同次批准；以前仅登包 MIT 是遗漏，现补登 | 两者潜在回退；已存在，文件哈希见 bundled-data-hashes.json |
| averaged_perceptron_tagger.zip | nltk_data 固定 550b6625bcef1f2abff2ff770a5a0d272c9c6b2a/packages/taggers | g2p_en 导入检查旧资源名，缺失会 nltk.download | 官方 XML MIT | 混合英文初始化，中文共享导入亦可能到达；固定 ZIP 和 XML 已哈希，保留 ZIP + 解压目录 |
| averaged_perceptron_tagger_eng.zip | 同 revision/packages/taggers | 当前 NLTK pos_tag 加载 JSON 英文标注器 | 官方 XML MIT | 混合；实际 NLTK perceptron.py 要求 _eng，不能只准备旧 pickle；准备状态见日志 |
| corpora/cmudict.zip（0.6） | 同 nltk_data revision/packages/corpora；官方 cmudict.xml | g2p_en 初始化 cmudict.dict；导入缺失会 nltk.download | 自定义 CMU 许可：研究和商业用途不受限制、希望署名；不是已核对 SPDX 家族，需用户批准 | 混合必需/共享英文初始化；未新增下载，阻塞 A |
| g2p_en/checkpoint20.npz、homographs.en | g2p_en 2.1.0 wheel，kyubyong/g2p | OOV 发音模型、同形词表 | 包随附 Apache-2.0；未找到独立更窄声明；按发行包声明登记，不声称训练数据权利已独立审计 | 混合；本地已有，逐文件哈希登记 |
| wordsegment/unigrams.txt、bigrams.txt、words.txt | wordsegment 1.3.1；前两者来自 norvig.com/ngrams 的 count_1w/count_2w | 英文复合词拆分/词频 | 包 Apache-2.0；Norvig 页面明确 MIT 指代码，数据来源 Google Web Trillion Word Corpus/LDC，独立数据授权范围未明确 | 混合；随包已有但阻塞 B，不按代码许可签发数据通过 |
| text/cmudict.rep | GPT-SoVITS 固定代码；CMUdict 0.07 文件头 | 英文词典 | 文件头 BSD 两条款，保留声明 | 混合；本地已有 |
| text/cmudict-fast.rep、engdict-hot.rep、namedict_cache.pickle | 固定 GPT-SoVITS 仓库 | 优化/热词/姓名词典 | 仓库 MIT；派生来源及独立数据授权未充分追溯 | 混合；阻塞 C，勿仅因根 LICENSE 就替所有派生数据定性 |
| text/opencpop-strict.txt、g2pw/polyphonic*.rep | 固定 GPT-SoVITS 仓库 | 拼音-音素、多音字覆盖 | 仓库 MIT；这些派生词表未找到逐文件来源许可说明，待统一确认 | 中文与混合；阻塞 C |
| text/engdict_cache.pickle、g2pw/polyphonic.pickle | 对上述词表运行时生成的本地缓存 | 加速词典加载 | 继承来源数据条件，不独立消除许可问题 | 两者对应路径；不作为原始来源 |
| G2PWModel/g2pW.onnx 与字符表/JSON | 已固定 XXXXRT/GPT-SoVITS-Pretrained revision；前轮 ZIP/哈希 | 中文多音字 | 模型仓库卡 MIT，保留前轮证据及边界 | 两者；已存在；目录缺失时 upstream onnx_api.py requests.get ModelScope master ZIP，禁网阻止此回退，不使用可变下载 |
| Chinese RoBERTa 权重/tokenizer/config | lj1995/GPT-SoVITS 固定 8f885...；已核验清单 | BERT 特征、G2PW tokenizer | 模型卡 MIT，按独立权重记录 | 两者；本地路径 from_pretrained；HF_HUB_OFFLINE + socket 禁网 |
| CNHuBERT 权重/preprocessor/config | 同固定模型 revision | 参考语音特征 | 模型卡 MIT | 两者；local_files_only=True，已预置 |
| s1v3.ckpt、s2Gv5turbo.pth、vocoder.pth | 同固定模型 revision | V5 TTS 推理 | 模型卡 MIT | 两者；完整大小/SHA 已通过 |
| BigVGAN.from_pretrained | TTS.py 中 v3 路线 | 旧版本 vocoder | 本次不下载/不判其权重许可 | 非 v5turbo 执行分支 |
| jieba/jieba_fast 的 dict、HMM/POS/finalseg/prob 数据 | 各已固定 wheel | 分词与词性 | 发行包 MIT，未发现单列限制；逐文件登记 | 两者共享文字路径，本地已有 |
| pypinyin 字典、cn2an 配置、OpenCC 字典/配置 | 各已固定 wheel | 拼音、数字规范化、简繁转换 | 分别发行包 MIT、MIT、Apache-2.0；逐文件登记 | 中文/混合本地数据；不触发下载 |
| resampy/data/*.npz | resampy 已固定 wheel | 重采样滤波系数 | 发行包 ISC | 两者音频路径，本地已有 |
| librosa 示例音频下载器/pooch | librosa.util.example 等 | 示例文件 | 本卡不使用，不下载 | 非推理调用；本卡参考由 VoiceDesign 合成、Apache-2.0 |
| torchmetrics / torchaudio pipelines / Lightning hub 工具 | 已安装包中的下载 API | 预训练指标、示例、hub/checkpoint | 非所用数据，未签发许可 | 静态存在但本次不调用；禁网守卫兜底 |
| NLTK downloader index、HF hub/requests/urllib 通用入口 | NLTK、transformers、huggingface_hub 等 | 上述缺失资源的解析/下载 | 必须按具体资源表核对，不能从库许可推导 | 在冒烟进程内禁止，目标地址写 blocked-network.jsonl |

## 一次性交回的未决项

A：批准或否决 NLTK CMUdict 0.6 的自定义宽松表述（官方 XML 已读取，未冒充 BSD SPDX）。
B：wordsegment 随包词频数据的独立授权范围需确认；Norvig 页仅明确代码 MIT，不足以单独证明数据也为 MIT。
C：上游派生热词/姓名/拼音/多音字词表的来源许可追溯不足；需用户决定接受固定仓库声明作为本机测试依据并在 P7 复审，或先要求补足来源证明。不是要求用户替第三方授予不存在的权利。

已批准 fastText 模型不再是阻塞。上述三组集中列出，未逐项启动真实推理试错。当前不得把静态盘点、已知包声明或禁网守卫自检说成 CPU 真实合成验收。

## 来源

- https://fasttext.cc/docs/en/language-identification.html ：模型 CC-BY-SA-3.0；署名 Armand Joulin、Edouard Grave、Piotr Bojanowski、Tomas Mikolov；Bag of Tricks for Efficient Text Classification / FastText.zip。
- https://creativecommons.org/licenses/by-sa/3.0/ ：模型许可声明，适用于独立 lid.176.bin/ftz；未修改模型。
- https://raw.githubusercontent.com/nltk/nltk_data/550b6625bcef1f2abff2ff770a5a0d272c9c6b2a/packages/corpora/cmudict.xml
- https://raw.githubusercontent.com/nltk/nltk_data/550b6625bcef1f2abff2ff770a5a0d272c9c6b2a/packages/taggers/averaged_perceptron_tagger.xml
- https://raw.githubusercontent.com/nltk/nltk_data/550b6625bcef1f2abff2ff770a5a0d272c9c6b2a/packages/taggers/averaged_perceptron_tagger_eng.xml
- https://www.norvig.com/ngrams/ ：代码与数据分别陈述；未据页面自行扩张许可。
