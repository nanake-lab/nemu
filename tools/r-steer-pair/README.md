# 中文两句话盲测：离线附件

当前规则见 [预注册 v2](../../reports/R-STEER-PAIR-01/PREREGISTRATION-V2.md)，[v1](../../reports/R-STEER-PAIR-01/PREREGISTRATION.md) 保留。实际题单已冻结，不要重跑抽样；用户现已批准v2并完成正式判断，本次授权的封存、解盲与分析已完成；见[结果报告](../../reports/R-STEER-PAIR-01/REPORT.md)。

## 本地验证

```powershell
node --check tools/r-steer-pair/app.js
node --check tools/r-steer-pair/prepare.mjs
node --check tools/r-steer-pair/browser-check.mjs
node --check tools/r-steer-pair/revise-v2.mjs
node --test tools/r-steer-pair/study.test.mjs tools/r-steer-pair/app.test.mjs tools/r-steer-pair/revise-v2.test.mjs
```

按用户本次审查意见，删除本卡专用 workflow 与 package.json 脚本；直接本地命令覆盖附件的关键边界，不新增产品运行时依赖，不修改通用 CI。

```powershell
# 先检查 Runner.Worker；独占浏览器检查窗口。仅使用合成夹具与已有依赖/浏览器。
node tools/r-steer-pair/browser-check.mjs <已有依赖的仓库根> <新检查输出目录> <已有Chromium绝对路径>
# 若计时逻辑变化，真实14分钟检查可加 timer 参数；当前 app.js 原样复用 v1。
```

浏览器使用已有 Playwright，禁用硬件 GPU；不自动安装、不联网。合成测试的结果不能当作人类判断。实际页面只检查首页，批准前不打开正式题目。

## 版本封存

`prepare.mjs` 是 v1 的历史准备入口，保留以供复核，**本次不执行**。`revise-v2.mjs` 只读取已冻结的 v1 匿名 HTML，核对原页面哈希，改版本、首页版本提示与显示对数；保留原题单、问法和作答代码。无原库、答案或密钥访问，无随机抽样。

```powershell
# 本轮已执行并封存；仅供复核，输出目录必须不存在。
node tools/r-steer-pair/revise-v2.mjs <已冻结v1页面绝对路径> <新输出目录>
```

v2 承诺绑定本版预注册哈希、两版页面哈希、相同题单与原答案承诺。页面和结果的版本为 v2；封存答案仍是 v1 的原表。交付仅复制 HTML 和使用说明；答案密文保存在私有证据，密钥独立留存，不推送、不交付。

批准 v2 前不得开始；正式完成后先封存最终结果，再获准解盲。历史准备工具不提供分析入口。本次用户已授权按v2解盲，新增 `analyze-v2.mjs`：验证先行封存的观察/结果及材料摘要，核验密文、解封答案承诺、题单与原输出哈希，再分类计数。密钥只在本机读取，不能推送或打印。明文答案与逐题分类仅保存私有证据，公开仅汇总。

```powershell
node --check tools/r-steer-pair/analyze-v2.mjs
node --test tools/r-steer-pair/analyze-v2.test.mjs
# 本轮已执行；现存分析目录会阻止覆盖，不要重新作答或重抽。
node tools/r-steer-pair/analyze-v2.mjs <公开任务仓绝对路径> <私有证据仓绝对路径> <本机custody绝对路径>
```

计数严格按v2：固定10个显示对、至少9个注入，质量/平局未支持；家族只报告。作答后的口头感受单列限制，不回填原始选择。
