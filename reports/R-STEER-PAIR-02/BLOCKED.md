# 当前阻塞

用户已批准本次GPU窗口。运行前只读预检发现Runner.Listener仍在运行，Runner.Worker未检出；未启动模型、GPU生成或API。

按AGENTS.md第9节及本卡预注册GPU窗口要求，runner存在时必须停止并报告。等待用户停掉runner后告知，再重新预检执行；不替用户停止进程，不绕过检查。也可由用户另约窗口。本次不是生成失败，无回答或条件结果。
