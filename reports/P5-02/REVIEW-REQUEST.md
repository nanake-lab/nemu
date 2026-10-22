实现 P5-02 / W02：单写 SQLite（WAL/FULL/FK）、CanonicalV1 内容寻址对象、R 倾向分段、完整版本绑定的暂存与原子发布，以及三种谱系的类型隔离。R、θ、L 和提交消息使用当前 core-types 生成类型，包括 P5-21 / P5-32 后继类型。

验收已完成：
- 同步最新 origin/main（91b31ed）；fmt、工作区 Clippy、77 项 Rust 测试通过，W02 为 23 项运行测试与 2 项编译拒绝测试。
- X03 覆盖 10 个真实进程退出点；另验证磁盘/容量拒绝、准备至提交期间 GC、旧数据库恢复拒绝、六个跨谱系打开方向及 P5-32 证据索引。
- 全部测试仅使用新建临时目录，未打开用户排练库或正式库，未做 GPU 测量。
- [Ubuntu CI](https://github.com/nanake-lab/nemu/actions/runs/39777111967) 与 [Windows CI](https://github.com/nanake-lab/nemu/actions/runs/39777926877) 均通过。后续证据提交不改变经测试的源文件树。

报告：[reports/P5-02/REPORT.md](reports/P5-02/REPORT.md)；逐项映射：[docs/traceability/P5-02.md](docs/traceability/P5-02.md)。保留初次编译、Clippy 失败日志及最终通过日志。

请 Claude 交叉审查，重点检查：对象文件及父目录持久化顺序、账本和状态的原子发布、恢复凭据与旧备份/双写限制、谱系隔离。账本与存储格式属于必审范围；审查通过前不合并。W14 的显式尾部恢复/有损分支登记、整设备回退的外部备份来源证明不在本卡实现范围；普通打开不会绕过这些要求。
