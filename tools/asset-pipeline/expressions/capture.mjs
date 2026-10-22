import { createServer } from 'vite';
import { chromium } from '@playwright/test';
import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { createRequire } from 'node:module';
const root = process.cwd();
const stageRequire = createRequire(resolve('apps/stage/package.json'));
const three = resolve(dirname(stageRequire.resolve('three')), '..');
const vrm = resolve(dirname(stageRequire.resolve('@pixiv/three-vrm')), 'three-vrm.module.js');
const server = await createServer({ root, configFile:false, optimizeDeps:{noDiscovery:true,include:[]}, resolve:{alias:[{find:'three/addons',replacement:three+'/examples/jsm'},{find:'three',replacement:three+'/build/three.module.js'},{find:'@pixiv/three-vrm',replacement:vrm}]},server:{host:'127.0.0.1',port:5187,strictPort:true}});
await server.listen();
const browser = await chromium.launch({args:['--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader']});
const output = resolve('reports/P2-06/visual'); await mkdir(output,{recursive:true});
const maps = JSON.parse(await readFile('content/_candidates/P2-06/emotionMap.candidates.json','utf8'));
const {customStrengths}=JSON.parse(await readFile('content/_candidates/P2-06/preview-catalog.json','utf8'));
const jobs = [];
for(const expr of ['nemuSmug','nemuCatMouth','nemuSparkle','nemuBlush','nemuTeary','nemuSleepy','nemuPout','nemuAngryMark'])
 for(const [variant,weight] of Object.entries(customStrengths)) jobs.push({expr,variant,weight,style:'standard',bg:'white',framing:'face'});
for(const emo of Object.keys(maps)) for(const variant of ['A','B']) jobs.push({emo,variant,style:'standard',bg:'white',framing:'face'});
for(const style of ['soft','standard','contrast']) for(const bg of ['white','black','gradient']) jobs.push({style,bg,framing:'upperBody'});
// Edge and overlay visibility on black are explicit supplementary views.
for(const expr of ['nemuSparkle','nemuBlush','nemuTeary','nemuAngryMark','nemuCatMouth']) jobs.push({expr,weight:1,style:'standard',bg:'black',framing:'face'});
const page = await browser.newPage({viewport:{width:440,height:500},deviceScaleFactor:1});
const errors=[];page.on('pageerror',e=>{errors.push(String(e));console.error('PAGE ERROR',e);});page.on('console',m=>{if(m.type()==='error')console.error('CONSOLE',m.text());});
const results=[];
try {
 for(const job of jobs.filter(j=>!process.env.P206_MATCH || new RegExp(process.env.P206_MATCH).test(JSON.stringify(j)))){
  const params = new URLSearchParams({deterministic:'1',...job});
  const url = `http://127.0.0.1:5187/tools/asset-pipeline/expressions/review.html?${params}`;
  const name = [...params].map(([k,v])=>`${k}-${v}`).join('_')+'.png';
  await page.goto(url);
  await page.waitForFunction(()=>window.__P206_READY__===true,null,{timeout:60000});
  const first=await page.screenshot();const second=await page.screenshot();
  if(!first.equals(second))throw new Error('Nondeterministic screenshot '+name);
  await writeFile(resolve(output,name),first);
  results.push({file:name,url,deterministic:true,...job});
  console.log(name);
 }
 if(errors.length)throw new Error(errors.join('\n'));
 if(process.env.P206_MATCH){const old=JSON.parse(await readFile(resolve(output,'screenshots.json'),'utf8'));const replaced=new Set(results.map(r=>r.file));results.push(...old.results.filter(r=>!replaced.has(r.file)));}
 await writeFile(resolve(output,'screenshots.json'),JSON.stringify({renderer:'SwiftShader software; candidate preview only',results,errors},null,2));
} finally {await browser.close();await server.close();}


