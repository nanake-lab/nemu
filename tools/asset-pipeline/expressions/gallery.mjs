import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { chromium } from '@playwright/test';
const dir=resolve('reports/P2-06');
const {results}=JSON.parse(await readFile(resolve(dir,'visual/screenshots.json'),'utf8'));
const names=['neutral','happy','relaxed','angry','worried','excited','sleepy','surprised','sad','shy','smug','pout'];
const chinese={neutral:'平静',happy:'开心',relaxed:'放松',angry:'生气',worried:'担心',excited:'兴奋',sleepy:'困倦',surprised:'惊讶',sad:'难过',shy:'害羞',smug:'得意',pout:'嘟嘴'};
const customs=['nemuSmug','nemuCatMouth','nemuSparkle','nemuBlush','nemuTeary','nemuSleepy','nemuPout','nemuAngryMark'];
const image=r=>`<figure><a href="visual/${r.file}"><img width="440" height="500" src="visual/${r.file}"></a><figcaption>${r.variant || ''} ${r.weight || ''} ${r.style} ${r.bg}</figcaption></figure>`;
let sections=[];
for(const expr of customs)sections.push(`<section id="${expr}"><h2>${expr} · A 0.55 / B 0.80 / C 1.00</h2><div class="row">${results.filter(r=>r.expr===expr&&r.bg==='white').map(image).join('')}</div><label>选择 <select name="${expr}"><option value="">待选</option><option>A</option><option>B</option><option>C</option><option>修改</option></select></label></section>`);
for(const emo of names){
 const i=names.indexOf(emo),group=['1-4','5-8','9-12'][Math.floor(i/4)],quadrant=i%4;
 const x=quadrant%2?100:0,y=quadrant>=2?100:0;
 const reference=`<figure><div class="reference" style="background-image:url('../../content/nemu/bible/02-expressions(${group}).png');background-position:${x}% ${y}%"></div><figcaption>设定集对应格 · ${chinese[emo]}</figcaption></figure>`;
 sections.push(`<section id="emo-${emo}"><h2>${emo} · ${chinese[emo]} · 设定集 / A / B</h2><div class="row">${reference}${results.filter(r=>r.emo===emo).map(image).join('')}</div><label>选择 <select name="emo-${emo}"><option value="">待选</option><option>A</option><option>B</option><option>修改</option></select></label></section>`);
}
for(const bg of ['white','black','gradient'])sections.push(`<section id="materials-${bg}"><h2>MToon · ${bg} · 柔和 soft / 标准 standard / 对比强 contrast</h2><div class="row">${results.filter(r=>!r.expr&&!r.emo&&r.bg===bg).map(image).join('')}</div></section>`);
const html=`<!doctype html><html lang="zh-CN"><meta charset="utf-8"><title>P2-06 表情与材质候选</title><style>*{box-sizing:border-box}body{margin:0;padding:20px;background:#ecebf0;color:#242331;font:16px system-ui}h1{margin:0}h2{margin:0 0 12px;font-size:21px}p{max-width:1100px}section{background:white;padding:18px;margin:20px 0;width:1356px;border-radius:10px}.row{display:flex;gap:0}figure{margin:0;width:440px}figure img{display:block;width:440px;height:500px}.reference{width:440px;height:294px;background-size:200% 200%;background-repeat:no-repeat}figcaption{padding:10px;font-size:16px}a{color:inherit}select,button{font:inherit;padding:7px}body.capture{padding:0}body.capture section{margin:0;border-radius:0}body.capture label,body.capture header,body.capture #export{display:none}textarea{width:900px;height:220px}</style><header><h1>P2-06 候选评审</h1><p>全部未定稿。8 个自定义表情各 3 档、12 种 emo 各 2 个组合；3 组材质在白、黑、彩色渐变背景下对照。A/B/C 表示同一实现的强度。emo 组合以当前 B/C 强度定义为基础展开，最终需按你的单项选择重新构建并签字。</p><p>预览使用 SwiftShader 软件渲染、固定镜头与光照，时间冻结。嘴型、眨眼和物理动态仍需在正式舞台复核。本页底模为 P2-05 粗短尾巴候选，未改动其文件。</p><p><a href="#emo-neutral">12 种组合</a> · <a href="#materials-white">3 组材质</a></p></header>${sections.join('')}<div id="export"><label>材质选择 <select name="materials"><option value="">待选</option><option>soft</option><option>standard</option><option>contrast</option><option>修改</option></select></label><p><button id="show">生成选择文本</button>（仅显示，未保存或批准）</p><textarea id="selection"></textarea></div><script>const panel=new URLSearchParams(location.search).get('panel');if(panel){document.body.classList.add('capture');for(const section of document.querySelectorAll('section'))if(section.id!==panel)section.remove();}document.getElementById('show').onclick=()=>{document.getElementById('selection').value=JSON.stringify(Object.fromEntries([...document.querySelectorAll('select')].map(s=>[s.name,s.value])),null,2)};</script></html>`;
await writeFile(resolve(dir,'review.html'),html);
const browser=await chromium.launch({args:['--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader']});
try{
 const page=await browser.newPage({viewport:{width:1356,height:620},deviceScaleFactor:1});
 await mkdir(resolve(dir,'sheets'),{recursive:true});
 for(const id of [...customs,...names.map(e=>'emo-'+e),...['white','black','gradient'].map(b=>'materials-'+b)]){
  await page.goto(pathToFileURL(resolve(dir,'review.html')).href+'?panel='+id);
  await page.waitForFunction(()=>[...document.images].every(i=>i.complete&&i.naturalWidth>0));
  await page.locator('section').screenshot({path:resolve(dir,'sheets',id+'.png')});
 }
}finally{await browser.close();}
console.log('23 comparison sheets rendered');
