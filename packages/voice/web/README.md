# 麦克风模块

```ts
import { startMic } from '@nemu/voice/web';
const mic = await startMic(microphonePort);
await mic.stop();
```

默认从当前页面目录加载 `asr-mic-worklet.js`。桌面构建与开发服务器由 `apps/desktop/electron.vite.config.ts` 自动提供该模块，不放宽舞台 CSP，不需要改动 apps/stage。其他宿主可将导出的 `workletSource()` 写到同源静态模块，并把该 URL 作为 startMic 的第二个参数。

默认不申请设备。主进程的 stage.mic.set 开启后才采集；MicController 管理异步权限、重复切换与关闭。每块严格使用 contracts 的 MicrophoneMessage：v=2、单调 seq、320 个 Int16 样本。Web 导出没有 Node、模型或云端依赖。
