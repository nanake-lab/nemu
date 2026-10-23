import { createServer } from 'vite';
import { chromium } from '@playwright/test';
import { readFile,writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
const root=process.cwd();
const server=await createServer({root:resolve(root,'apps/stage'),configFile:resolve(root,'apps/stage/vite.config.ts'),server:{host:'127.0.0.1',port:5185,strictPort:true,fs:{allow:[root,'D:/Dev/nemu/content']}}});
server.middlewares.use(async(req,res,next)=>{const paths={'/p205-source.vrm':'D:/Dev/nemu/content/nemu/source/nemu-vroid-export.vrm','/p205-source.profile.json':resolve(root,'tools/asset-pipeline/.cache/source.profile.json')}; const file=paths[req.url];if(!file)return next();res.end(await readFile(file));});
await server.listen();
const browser=await chromium.launch({headless:true,args:['--ignore-gpu-blocklist','--use-gl=angle']});
try{
const page=await browser.newPage({viewport:{width:1440,height:1050},deviceScaleFactor:1});
page.on('console',m=>{if(m.type()==='error')console.log('CONSOLE',m.text());});
page.on('pageerror',e=>console.log('PAGEERROR',String(e)));
await page.goto('http://127.0.0.1:5185/review.html?deterministic=1');
await page.waitForFunction(()=>window.__REVIEW_READY__===true);
await page.evaluate(async()=>{const a=window.__reviewApp;a.stop();await a.loadModel('/@fs/D:/Dev/nemu-p205/tools/asset-pipeline/.cache/v3-standard-standard/nemu.vrm','/@fs/D:/Dev/nemu-p205/tools/asset-pipeline/.cache/v3-standard-standard/nemu.profile.json');a.stop();if(a.currentVrm.meta.name!=='Nemu')throw new Error('Expected Nemu, loaded '+a.currentVrm.meta.name);a.currentVrm.humanoid.resetNormalizedPose();a.currentVrm.humanoid.update();a.currentVrm.springBoneManager.reset();a.turntable.setPreset('front',false);a.turntable.setDistance(5.3);a.setBackgroundPreset('white');a.render();});
await page.screenshot({path:resolve(root,'reports/P2-05/v3-standard-front.png')});
await page.evaluate(()=>{const a=window.__reviewApp;a.turntable.setPreset('back',false);a.render();});
await page.screenshot({path:resolve(root,'reports/P2-05/v3-standard-back.png')});
const state=await page.evaluate(()=>{const a=window.__reviewApp;const r=a.renderer.renderer;return {loaded:a.currentVrm.meta,profile:a.currentProfile};});
await writeFile(resolve(root,'reports/P2-05/v3-standard-preview-state.json'),JSON.stringify(state,null,2));
}finally{await browser.close();await server.close();}
