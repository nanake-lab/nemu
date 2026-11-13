# R-STEER-PAIR-01 离线准备工具

方案见 [预注册](../../reports/R-STEER-PAIR-01/PREREGISTRATION.md)。只构建已有输出的匿名页面；没有生成、评分、联网、GPU 或结果分析入口。

```powershell
node --check tools/r-steer-pair/app.js
node --check tools/r-steer-pair/prepare.mjs
node --test tools/r-steer-pair/study.test.mjs
# 一次性准备；两个新目录不得已存在。不要因质量筛除重跑。
node tools/r-steer-pair/prepare.mjs <私有源仓绝对路径> <新证据输出目录> <独立新custody目录>
```

交付目录只复制 `comparison.html`。`answers.sealed.json` 归私有证据；custody 不推送，不交付，不链接给参与者。公开仅复制 `commitment.json`。后续解封须先封存用户最终结果并获得确认，当前没有解封/分析命令。

正常 CI 运行匿名合成夹具测试及语法检查。浏览器检查只用夹具；实际材料只做结构与来源核验，不替用户作答。没有新增运行时依赖（Node 内置模块；浏览器检查使用仓库已有 Playwright）。
