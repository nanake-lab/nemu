# Base 完整下载交接与 CPU 转换准备

用户于 2026-11-08 批准原卡继续，包括重新设计并预注册测量接口；明确不再下载 Base。继续时先 fetch，并将 `origin/main` `a7e8ae8790d297ac697956f6f1256d2e2b9403d5` merge 入任务分支，合并提交 `bba90f48`。更新仅涉及 P5-39 场景和看板；AGENTS、P5-31、载体通路未变化。C0 的原 main 基线与实际运行提交保持原记录，不能改写成这次的 main。

## 交接实测核验

输入为 `D:/Dev/cache/R-STEER-01/base-download-03/model`、同目录上级的 inventory/result/metadata/events，以及缓存根的下载程序和 launch/stdout/stderr。目标仓库 `Qwen/Qwen3.5-4B-Base`，固定修订 `1001bb4d826a52d1f399e183466143f4da7b741b`，许可声明 Apache-2.0。

机械核验逐文件重算：13 个文件、9,342,824,751 字节，13/13 字节数和 SHA-256 与 inventory 一致。保留的官方 metadata 中三个 LFS SHA-256（两片权重及 tokenizer）和十个 Git blob SHA-1 均与实物一致。本次没有重新联网核验 metadata 的来源，也没有重新下载。主代理检查核验命令退出 0、完整逐文件结果、路径集合及事件终态后接受该交接。

| 权重 | 字节 | SHA-256 |
| --- | ---: | --- |
| 00001-of-00002 | 5,329,398,712 | `df547074dce70532a0493e5433152bd17a65efb89088cfabc2e7e2371a93d712` |
| 00002-of-00002 | 3,990,429,344 | `590fbaac095dd31db886c322d9d2f7df47777966391acf306ddddc3e4e3a15ef` |

下载结果 complete，UTC `2026-11-08T15:17:32.546342+00:00`。247 次 range_attempt 中有四次部分传输失败（三次 curl 28、一次 curl 56）；每次的错误、partial、后继成功范围记录都保留。后继起点等于首次起点加收到字节，终点不变、HTTP206，最终完整文件经过全量哈希。这些失败不因完成下载而被删去。更早的900秒 curl28和另批30分钟 curl56失败仍在原归档及缓存中。

下载程序 `download_complete_03.py` SHA-256 `937754b7ec36e2f35788a58258b3029edac150229fcdbcd64a3483391ea74e5b` 与 launch 记录一致。交接程序、命令、元数据、原始日志与核验结果无损归档于 [base-handoff-03](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/base-handoff-03/raw-filemap.json)，大权重及范围 partial 留在 D 盘缓存。没有模型推理。

核验按 AGENTS §9 委派机械代理；读取项目配置，显式参数试用 `gpt-6.1-terra/max`，不是已验证的按角色名自动加载。代理只输出证据，没有判定验收、修改代码或推送。

## 转换实施

新脚本 [convert_base03.ps1](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/convert_base03.ps1) 只读取完整的 `base-download-03/model`，核对 complete、精确文件集合、修订、字节数、全部模型 SHA、固定源码清单和 CPU 量化器文件；旧 `prepare_base.ps1` 冻结为历史准备，不能运行它的旧路径来转换。

沿 P0-12 `convert_visual_model.ps1`：源码 b11669/`620d2225cc436ab0b001e3200e590e3b286ef9ab`，现有 CPU venv；`--outtype bf16 --no-mtp`，`Q4_K_M 4`，mmproj `--outtype f16`。离线、CUDA隐藏、CPU torch、4线程，与 P0-12 一致；只对子 pwsh 设置环境。输出新建 `base-gguf-03`，日志 `base-convert-03`；存在即拒绝覆盖、非零退出即保留并停止。保留 argv、工作目录、UTC、退出码、实际输入/工具与产物哈希。CPU 转换不等于 Base 行为已测试，也不需要或沿用 GPU 窗口。

## 转换结果（Perth 11月8日23:45–23:48；11月9日整理）

第一次`base-convert-03`在环境记录步骤退出1：执行者误把源码内置gguf当成pip distribution，`importlib.metadata.version("gguf")`报PackageNotFoundError。CPU torch断言已通过，但三条转换均未开始，原环境日志、failure、外层日志保留。确认固定转换器第14–15行自行加载`gguf-py`后，新脚本 [convert_base04.ps1](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/convert_base04.ps1)只移除这一错误的版本查询，使用新输出/证据目录；不安装依赖、不换转换器或参数、不覆盖03。源码内gguf全部由189文件源码清单绑定。

`base-convert-04`四条命令均退出0，外层结束`2026-11-08T15:48:44.6289247+00:00`。Python3.12.10、torch2.11.0+cpu、transformers4.57.6。BF16转换、Q4量化、mmproj转换的参数与P0-12一致；量化用4线程，约93.86秒。只完成离线CPU准备，Base推理`not_run`。

| 产物（位于D:/Dev/cache/R-STEER-01/base-gguf-04） | 字节 | SHA-256 |
| --- | ---: | --- |
| carrier-BF16.gguf | 8,424,393,376 | `d08780fb499980b57ba7072914b62dcebd88364a51213507f3de09bcbc878776` |
| carrier-Q4_K_M.gguf | 2,708,804,256 | `8d00d56989d20b50740f57bfa4ebb5ce9bcc6279d7ce13d0551eebf70de2b155` |
| mmproj-F16.gguf | 672,423,136 | `f1d07963998492d7d380e826635141afcf2e7e7d544f1a8e1ef389d4065b9a19` |

主脚本各次均核对13模型、189源码、30CPU工具文件，另记录5条脚本/解释器/清单摘要（237条）；独立机械核对重新哈希三个GGUF、converter及全部30CPU工具，与结果/固定清单一致。独立步骤没有重复读取输入9.34GB权重，它核对两次实测记录和已完成的交接全量核验；不把记录比对写成又一次全量输入复算。converter SHA256 `e9a1da876330bbce9687541ab31736542a01b4ac43c6686126514a50f122fb7f`。

全部argv、cwd、UTC、退出码、原始成功/失败日志、工具清单、GGUF清单和独立核验程序/输出见 [无损转换归档](https://github.com/nanake-lab/nemu-evidence/blob/67bd1831d34cbc533ad354d46513b7ad56c1100d/R-STEER-01/base-convert-04/raw-filemap.json)。GGUF和权重留D盘缓存，不提交大模型。没有模型或引擎改动、全局设置或新依赖。
