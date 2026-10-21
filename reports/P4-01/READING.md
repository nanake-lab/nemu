# P4-01 日常朗读录音表

状态：待用户录音。每种语言 20 句，各句只读一遍，自然语速；中文第 19–20 句按中文为主要语言。

本机目录：`%LOCALAPPDATA%\Nemu\asr-evaluation\user\<zh|en|ja>\`，可用 `NEMU_ASR_DATA` 改父目录。文件名 `001.wav` 至 `020.wav`，对应本表编号。建议单声道 PCM WAV，句首、句尾各留约一秒安静；不用剪掉停顿。保留原始录音，采样率转换由评测脚本完成，不覆盖原文件。

运行 `pnpm asr:user` 先建立目录及每句的 `001.txt` 等文字稿；录完后再次运行同一命令登记音频 SHA-256，然后运行 `pnpm eval:asr`。缺句会逐条列出；未齐 60 条时不能签发 150 条验收。若实际读法改变，请在相邻 txt 里如实改稿，不要按识别结果修改参考稿。

录音、用户文字稿和 user.json 均保留在仓库外，不提交 Git。评测报告只保存用户样本的哈希、语言与错误计数，不保存录音或识别全文。

| 编号 | 中文 zh | English en | 日本語 ja |
| --- | --- | --- | --- |
| 001 | 今天的天气很舒服。 | Good morning, how are you today? | おはよう、今日はいい天気だね。 |
| 002 | 我想先喝一杯温水。 | I would like a glass of water. | 少しだけ音を小さくして。 |
| 003 | 帮我看看现在几点了。 | Please turn the volume down a little. | お水を一杯飲んでから始めよう。 |
| 004 | 等我五分钟再开始。 | Give me five minutes to finish this. | あと五分だけ待っていてね。 |
| 005 | 今天晚上我想早点休息。 | I am going to take a short break. | 今日は早めに休もうと思う。 |
| 006 | 请把声音调小一点。 | Could you say that one more time? | もう一度ゆっくり話してくれる？ |
| 007 | 我刚刚把窗户打开了。 | The weather is lovely this afternoon. | 明日は傘を持っていこう。 |
| 008 | 明天上午记得带雨伞。 | I saved the document on my desktop. | 今は会議中だから静かにしてね。 |
| 009 | 我想听听你的想法。 | Let us talk about this after lunch. | このファイルは保存しておいたよ。 |
| 010 | 先别说话，我在开会。 | Please stay quiet while I am in a meeting. | お昼ご飯のあとで相談しよう。 |
| 011 | 这件事情我们慢慢来。 | I think I left my keys on the table. | 机の上の本はどこにある？ |
| 012 | 我已经把文件保存好了。 | What time should we leave tomorrow? | ちょっと外を散歩してくるね。 |
| 013 | 桌上的那本书在哪里？ | I would like to hear your opinion. | あなたの考えも聞いてみたい。 |
| 014 | 今天的午饭很好吃。 | There is no need to hurry today. | 急がなくても大丈夫だよ。 |
| 015 | 等一下，我还有一个问题。 | I will be back in about ten minutes. | あと一つだけ質問してもいい？ |
| 016 | 请用中文再说一遍。 | That was a really helpful suggestion. | 今日はとても助かったよ。 |
| 017 | 我准备出去散步十分钟。 | Nemu, could you help me check this? | ネム、少し休憩しようか。 |
| 018 | Nemu，今天辛苦了。 | I want to get some sleep early tonight. | 明日の朝は何時に出かける？ |
| 019 | 这个 meeting 我们下午再聊。 | Please remind me to bring my umbrella. | 窓を開けてもいいかな。 |
| 020 | 先把这个 task 放到明天。 | We can finish the rest of the work tomorrow. | 残りの作業は明日にしよう。 |
