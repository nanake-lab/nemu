# 机械只读审计留存

主代理按 routine_reporter 角色名调度，读取项目角色配置；配置声明 gpt-6-luna/max，但角色文件自动加载本身未独立验证。无 GPU、模型加载、测试或构建。

有效数据检查点为 stage1-checkpoint.json（延迟/显存/通路）、stage2-v3-checkpoint.json（norm/81 条剂量）、stage3-v3-checkpoint.json（HTML）。主代理核对返回值与原始测量：三次 p50/p90、WDDM owned counters 两项最大值之和、allocator 峰值、六个 norm 中位数、固定系数及 U/plan 字节 SHA 全部一致。HTML 81 行逐条匹配；本次输出无 <>&，特殊字符转义另由既有 CPU 测试覆盖。

失败原样留存，不将其写成测量失败或全部命令成功：首次内联审计误读 direction.direction_norm（实际在 norms.json）而 TypeError，Python 退出 1、外层 PowerShell 退出 0，当时没有 stdout/stderr 文件，工具记录的错误副本在 prior-inline-failure-copy.json。stage2 第一次又因 tuple 键不可 JSON 序列化失败；stage2-checkpoint.json 是不完整 JSON，不应当作有效检查点。stage2-v2-checkpoint.json 保留错误字段子检查；修正后的 v3 是有效结果。audit.py / audit-v2.py / audit-v3.py 为临时机械核验原脚本副本（约 400 行以上仅因整段审计证据原样保留，不是运行时模块），不参与应用或 CI。成功阶段 stderr 为空。

run-1 的最大采样间隔 6.015322 秒发生在 memory.jsonl 第 1/2 行起点 15:41:17.3738512Z 与 15:41:23.3891731Z；第二行早于 path 记录 37.944103 秒、早于 render 43.225827 秒、早于 concurrent 加载前记录 46.260178 秒。没有把此处间隔隐去或推断成连续峰值。

m=0 9 条：EOS 6、token_limit 3。剂量 81 条：EOS 56、token_limit 25；最大 token 48，最大 elapsed_s 2.8430159，无 time_limit，EOS 均在末尾。HTML 1 表，82 tr 含表头。变化计数只是 token 序列差，不等于方向、语义或能力验证。
