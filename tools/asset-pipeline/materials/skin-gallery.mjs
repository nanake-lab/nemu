import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { chromium } from '@playwright/test';
import { assertQuiet } from './audit-capture.mjs';
const dir = resolve('reports/P2-06/skin-audit');
const { results } = JSON.parse(await readFile(resolve(dir, 'visual/screenshots.json'), 'utf8'));
const label = {
  control: '对照 · 原 standard',
  A: 'A 暖杏 · 弱轮廓光',
  B: 'B 蜜桃 · 中轮廓光',
  C: 'C 暖金 · 强轮廓光',
};
const dirs = { left: '主光在画面左侧', right: '主光在画面右侧' };
let sections = [];
for (const framing of ['face', 'upperBody'])
  for (const light of ['left', 'right'])
    for (const bg of ['white', 'black', 'gradient']) {
      const items = results.filter(
        (r) =>
          r.config.framing === framing && r.config.light === light && r.config.background === bg,
      );
      sections.push(
        `<section id="${framing}-${light}-${bg}"><h2>${framing === 'face' ? '脸部近景' : '上半身'} · ${dirs[light]} · ${bg}</h2><div class="row">${items.map((r) => `<figure><figcaption>${label[r.config.variant]}</figcaption><a href="visual/${r.file}"><img width="440" height="500" src="visual/${r.file}"></a></figure>`).join('')}</div></section>`,
      );
    }
const html = `<!doctype html><html lang="zh-CN"><meta charset="utf-8"><title>P2-06 暖肤色候选</title><style>*{box-sizing:border-box}body{margin:0;padding:20px;background:#efedf0;color:#27232b;font:17px system-ui}header{max-width:1200px}section{padding:18px;background:white;width:1796px;margin:24px 0;border-radius:10px}h1{margin:0}h2{margin:0 0 12px;font-size:22px}.row{display:flex}figure{margin:0;width:440px}img{display:block;width:440px;height:500px}figcaption{padding:10px;font-size:18px}a{color:#4a4470}p{line-height:1.65}body.capture{padding:0}body.capture section{margin:0}body.capture header,body.capture footer{display:none}.diagnostic{display:flex;gap:10px}.diagnostic img{width:330px;height:375px}select,button,textarea{font:inherit;padding:9px}</style><header><p style="padding:12px;background:#fff0cf">2026-10-23 用户已选择：保留原版皮肤对照；不采用 A/B/C，暂不改底色。以下对照保留作为证据。</p><h1>P2-06 脸部与皮肤：暖色候选</h1><p>三组均未定稿。只改脸、身体皮肤的阴影色、明暗交界与参数化轮廓光。所有底色因子、贴图字节和非皮肤材质保持不变。四列始终为原 standard 对照 / A / B / C；全部使用正式舞台的输出类、LinearToneMapping、曝光 1，左右仅镜像灯光方向。</p><p>这次先隔离灯光差异：旧候选预览的主光 2.1、环境光 0.65 高于舞台 1.1、0.45。本页没有通过错误标记贴图色彩空间或给底色染色来制造暖感。<a href="../SKIN-COLOR-AUDIT.md">查看色彩审计与底色处理选项</a> · <a href="#upperBody-left-white">查看上半身</a> · <a href="../review.html">返回原表情候选</a></p><h2>诊断对照：同机位，材质/灯光分开看</h2><div class="diagnostic">${[
  ['legacy-no-tone', '旧候选灯光 + standard'],
  ['stage-standard', '舞台灯光 + standard'],
  ['stage-source', '舞台灯光 + 源材质'],
  ['source-base-unlit', '源底色无光照诊断'],
]
  .map(([f, l]) => `<figure><figcaption>${l}</figcaption><img src="${f}.png"></figure>`)
  .join(
    '',
  )}</div></header>${sections.join('')}<footer><p>原底色是很浅的桃肤色。若想让受光面也更暖/更有血色，另选底色处理路线：VRoid 另存新源版本重新导出；或扩展 texture-fix 做皮肤 UV 蒙版内的连续调色。两条路线均未执行，也不要求手绘。</p><label>肤色候选 <select id="pick"><option selected>保留原版对照（已选）</option><option>A 暖杏</option><option>B 蜜桃</option><option>C 暖金</option><option>继续调整</option></select></label><label>底色路线 <select id="route"><option>暂不改底色</option><option>VRoid 新版本</option><option>texture-fix 蒙版调色</option><option>先讨论</option></select></label><p><button id="show">生成选择文本</button>（仅显示，不保存或批准）</p><textarea id="selection" rows="4" cols="80"></textarea></footer><script>const panel=new URLSearchParams(location.search).get('panel');if(panel){document.body.classList.add('capture');for(const s of document.querySelectorAll('section'))if(s.id!==panel)s.remove();}document.getElementById('show').onclick=()=>document.getElementById('selection').value='肤色：'+document.getElementById('pick').value+String.fromCharCode(10)+'底色路线：'+document.getElementById('route').value;</script></html>`;
await writeFile(resolve(dir, 'review.html'), html);
await mkdir(resolve(dir, 'sheets'), { recursive: true });
assertQuiet();
const browser = await chromium.launch({
  args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'],
});
try {
  const page = await browser.newPage({
    viewport: { width: 1796, height: 620 },
    deviceScaleFactor: 1,
  });
  for (const framing of ['face', 'upperBody'])
    for (const light of ['left', 'right'])
      for (const bg of ['white', 'black', 'gradient']) {
        const id = `${framing}-${light}-${bg}`;
        await page.goto(pathToFileURL(resolve(dir, 'review.html')).href + '?panel=' + id);
        await page.waitForFunction(() =>
          [...document.images].every((i) => i.complete && i.naturalWidth > 0),
        );
        await page.locator('section').screenshot({ path: resolve(dir, 'sheets', id + '.png') });
      }
} finally {
  await browser.close();
}
console.log('12 skin comparison sheets rendered');
