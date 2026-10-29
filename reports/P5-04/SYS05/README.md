# SYS05

测试位于本卡允许的 fact-view/src/tests/mod.rs：sys05_five_distinct_results_and_real_positive_control。

新建 TemporaryRoot EXPERIMENT：先提交合成事实，验证能成功返回；分别查询不存在文字、撤销授权、条数预算为 0、提交新事实使派生索引落后、在已验证重建后删除临时载荷对象。面板适配依次得到五种不同的生成 QueryStatus，而非同一个成功空集。对象删除前断言路径位于拥有的 TemporaryRoot 内。

result.json 与 workspace-tests-acceptance.log 记录运行结果。真实向量撤回和三语召回另见 ../recall/；存储前缀恢复核验及谱系编译失败用例见同一完整工作区日志。
