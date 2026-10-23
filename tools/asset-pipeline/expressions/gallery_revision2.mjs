import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { chromium } from '@playwright/test';
import { assertQuiet } from '../materials/audit-capture.mjs';
const dir = resolve('reports/P2-06/revision2');
const { results } = JSON.parse(await readFile(resolve(dir, 'visual/screenshots.json'), 'utf8'));
const catalog = JSON.parse(await readFile('content/_candidates/P2-06/revision2/preview-catalog.json', 'utf8'));
const chinese = { neutral:'平静', happy:'开心', relaxed:'放松', angry:'生气', worried:'担心', excited:'兴奋', sleepy:'困倦', surprised:'惊讶', sad:'难过', shy:'害羞', smug:'得意', pout:'嘟嘴' };
const customs = { nemuSmug:'得意', nemuCatMouth:'轻猫嘴', nemuSparkle:'星星眼（沿用 A）', nemuBlush:'脸红（沿用 C）', nemuTeary:'单侧大泪珠', nemuSleepy:'闭眼困倦', nemuPout:'小巧嘟嘴', nemuAngryMark:'生气与怒符' };
const names = Object.keys(chinese);
const prefill = { nemuSparkle:'A', nemuBlush:'C', 'emo-neutral':'B', 'emo-happy':'B', 'emo-relaxed':'A', 'emo-excited':'B', materials:'contrast' };
const esc = value => String(value).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('"', '&quot;');
const select = (name, options) => `<label>本轮选择 <select name="${name}"><option value="">待选</option>${options.map(v => `<option>${v}</option>`).join('')}<option>修改</option></select></label>`;
const image = r => `<figure><figcaption>${esc(r.variant)} · ${esc(r.bg)}</figcaption><a href="visual/${r.file}"><img width="440" height="500" src="visual/${r.file}"></a></figure>`;
const sections = [];
const diagnosticImage = (file, title) => `<figure><figcaption>${title}</figcaption><img width="440" height="500" src="visual/${file}"></figure>`;
sections.push(`<section id="mouth"><h2>嘴下阴影线：来源与去除候选</h2><p>这是下唇下方凹槽的明暗线，关闭描边仍在；不属于下巴外轮廓。候选仅平滑这一小块的表面法线，保留全部顶点位置、UV、底图和下巴形状。</p><div class="row">${diagnosticImage('deterministic-1_revision-r1_variant-B_diagnostic-control_bg-white_framing-face.png','处理前')}${diagnosticImage('deterministic-1_revision-r1_variant-B_diagnostic-all-outline_bg-white_framing-face.png','关闭全部描边：线仍在')}${diagnosticImage('deterministic-1_variant-B_diagnostic-lip-smoothed_bg-white_framing-face.png','局部平滑候选')}</div></section>`);
for (const [name, label] of Object.entries(customs)) {
  const kept = name === 'nemuSparkle' ? 'A' : name === 'nemuBlush' ? 'C' : null;
  const items = results.filter(r => r.expr === name && r.bg === 'white' && (!kept || r.variant === kept));
  sections.push(`<section id="${name}"><h2>${label} · ${name}</h2><p>${esc(catalog.intent[name] || '沿用你在上一轮已选的强度，统一舞台灯光重新展示。')}</p><div class="row">${items.map(image).join('')}</div>${select(name, kept ? [kept] : ['A','B','C'])}</section>`);
}
for (const emo of names) {
  const i = names.indexOf(emo), group = ['1-4','5-8','9-12'][Math.floor(i/4)], q = i%4;
  const reference = `<figure><figcaption>原设定参考 · 可推翻</figcaption><div class="reference" style="background-image:url('../../../content/nemu/bible/02-expressions(${group}).png');background-position:${q%2 ? 100:0}% ${q>=2 ? 100:0}%"></div></figure>`;
  sections.push(`<section id="emo-${emo}"><h2>${chinese[emo]} · ${emo}</h2><div class="row">${reference}${results.filter(r => r.emo === emo).map(image).join('')}</div>${select('emo-'+emo,['A','B'])}</section>`);
}
sections.push(`<section id="materials"><h2>整体材质：沿用 contrast，皮肤保留原对照</h2><p>此次没有更改肤色底色或贴图。当前组合在三个背景下展示；上轮 soft / standard / contrast 对照仍可在历史页查看。</p><div class="row">${results.filter(r => !r.expr && !r.emo).map(image).join('')}</div>${select('materials',['soft','standard','contrast'])}</section>`);
const data = JSON.stringify({ revision:catalog.revision, fingerprint:catalog.fingerprint, prefill }).replaceAll('<','\\u003c');
const html = `<!doctype html><html lang="zh-CN"><meta charset="utf-8"><title>P2-06 第二轮表情评审</title><style>*{box-sizing:border-box}body{margin:0;padding:20px 20px 150px;background:#ecebf0;color:#28232b;font:16px system-ui}h1{margin:0}p{line-height:1.65;max-width:1120px}section{width:1356px;padding:18px;background:white;border-radius:10px;margin:24px 0}.row{display:flex;align-items:flex-start}figure{width:440px;margin:0}figcaption{padding:10px}img{display:block;width:440px;height:500px}.reference{width:440px;height:294px;background-size:200% 200%;background-repeat:no-repeat}select,button,textarea{font:inherit;padding:9px}label{display:block;margin:14px 0}textarea{width:100%;max-width:1100px}a{color:#574273}#actions{position:fixed;bottom:0;left:0;right:0;background:#fffef8;padding:12px 20px;border-top:1px solid #bcb3c6;z-index:3;box-shadow:0 -2px 10px #0001}#submit{background:#574273;color:white;border:0;border-radius:6px}#save-status{margin-left:16px;font-size:14px}body.capture{padding:0}body.capture section{margin:0}body.capture header,body.capture label,body.capture #actions,body.capture #feedback{display:none}</style><header><h1>P2-06 第二轮：表情重做</h1><p>已保留下来的选择：星星眼 A、脸红 C、平静 B、开心 B、放松 A、兴奋 B、整体 contrast。受本轮修改影响的项目等待重新选择；沿用的组合偏好也请看新图后提交。worried 上次选了 B，但按你的新反馈继续修改泪珠。</p><p>Smug 改用嘴角表达得意；猫嘴降幅；泪珠单侧放大；困倦全闭眼；嘟嘴收窄；怒符改圆角并加入生气眼神。原设定格只作参考，本轮以你的反馈为准。所有图使用舞台灯光、固定机位和时间冻结，旧图保留。<a href="../review-r1.html">历史第一轮</a> · <a href="../SKIN-COLOR-AUDIT.md">肤色审计</a> · <a href="#emo-neutral">12 种组合</a></p><p>提交会保存到本机候选目录并给出回执；不会自动定稿或合并。改动会自动保存为浏览器草稿，亦可导出文件备份。</p></header>${sections.join('')}<section id="feedback"><label for="notes">补充修改意见</label><textarea id="notes" rows="5" maxlength="8000" placeholder="例如：选 B，但嘴角再轻一点"></textarea></section><div id="actions"><button id="submit">提交本轮选择 / 修改</button> <button id="download">导出草稿</button><span id="save-status" role="status" aria-live="polite">尚未提交 · 已沿用上轮未要求修改的偏好</span></div><script type="application/json" id="review-data">${data}</script><script type="module" src="/tools/asset-pipeline/expressions/review-form.mjs"></script><script>const panel=new URLSearchParams(location.search).get('panel');if(panel){document.body.classList.add('capture');for(const s of document.querySelectorAll('section'))if(s.id!==panel)s.remove();}</script></html>`;
await writeFile(resolve(dir,'review.html'), html);
if (!process.argv.includes('--html-only')) {
  assertQuiet();
  await mkdir(resolve(dir,'sheets'), { recursive:true });
  const browser = await chromium.launch({ args:['--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader'] });
  try {
    const page = await browser.newPage({ viewport:{width:1356,height:640}, deviceScaleFactor:1 });
    for (const id of ['mouth',...Object.keys(customs),...names.map(n=>'emo-'+n),'materials']) {
      await page.goto(pathToFileURL(resolve(dir,'review.html')).href+'?panel='+id);
      await page.waitForFunction(() => [...document.images].every(i => i.complete && i.naturalWidth));
      await page.locator('section').screenshot({ path:resolve(dir,'sheets',id+'.png') });
    }
  } finally { await browser.close(); }
}
