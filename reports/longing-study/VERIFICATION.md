# 核验与复现

本目录是设计研究；没有测试或修改产品实现，不将研究脚本结果写成 AC2/I23/CI 通过。

## 执行

工作目录：/workspace/scratch/2c4129af8a1f/longing-study。
命令（也可在 Windows 仓库根目录 PowerShell 执行）：

    python reports/longing-study/simulate.py --out reports/longing-study/evidence/run-01

首次模拟退出 0。目标目录必须不存在；复现时改用一个新输出目录，不能覆盖原始证据。
Python 版本、脚本 SHA-256 与参数见 evidence/run-01/manifest.json。
只有 Python 标准库，无新增运行时依赖、无网络、无模型调用、无真实数据。

## 已完成的检查

- 原始输入带、CSV、JSON、SVG 的 6 个清单哈希及脚本哈希全部一致。
- CSV 数据行数：trajectories 720，paired_paths 360，feedback_sensitivity 605。
- 430 次记录带重放逐位一致；解析保存的 binary64 十六进制输入，不重新生成内容。
- 等间隔同号脉冲与闭式公式最后值误差为 0；有限/盒界、停机零事件断言通过。
- offline_gap 的 step 11–90 有 80 个仅供画图的投影视点，实际提交事件为 0。
- 固定带收缩界最大浮点超量 5.5312341686215506e-17 原样保留。
- 反例轨迹完整保留：同事件自激最终差 2；状态相关选题最终差 1.1033311131845016。
- SVG 可解析为合法 XML；图中的单位为抽象时间与有符号坐标。
- 研究脚本 190 行以内，报告不足 400 行。无大型文件或依赖。
- 本地所有交付文件在 reports/longing-study/；远端另按 base/head 比较核对路径、草稿及标签。

## 失败与限制

机械核验子代理首次误把清单中的文件名相对于报告根目录解释，FileNotFoundError，退出 1；改为相对于 evidence/run-01 后哈希/行数检查退出 0。该核验还曾使用错误的 condition 列名，后按实际 scenario 字段检查到 80 行，0 事件；没有修改模拟、数据或阈值。主代理独立重复了哈希/行数/停机检查，退出 0。

主代理提前检查报告本地链接时，VERIFICATION.md 尚未创建，AssertionError，退出 1；补齐本文件后再次检查。不属于数值实验失败，也没有删改原始输出。失败信息见 evidence/verification-attempts.txt。

Git HTTPS 读取因环境没有凭据失败，Windows D 盘不可访问；因此采用 GitHub 连接读取固定提交及发布研究，未完成指定的 Windows worktree。没有用系统设置、浏览器登录、密钥查找或网络规避来弥补。

本脚本不包含真实 G/C、R/θ 学习、语义评估、CanonicalV1、thinning、完整闭环策略或 Nemu 时钟/事务；记录带重放示例不能替代 I16/I23，有限轨迹不能证明所有参数与模型选题下稳定。未来完整验收未执行，本次也不需要跑应用 CI。

## 原始文件

- input_tapes.json：所有实际非零脉冲事件及其十六进制输入/结果；0 输入时没有回想事件。
- trajectories.csv：全部 6 种情景，每个采样时刻的输入、投影与执行状态。
- paired_paths.csv：固定带、旧式自激、状态相关选题的所有配对状态。
- feedback_sensitivity.csv：5 种反馈增益下的解析连续轨迹；保留未截断数值。
- summary.json：汇总、闭式校验与证明适用条件。
- trajectories.svg：由同批采样作图，没有删除不利情景。
- manifest.json：输入参数、计算环境、脚本与输出哈希。

