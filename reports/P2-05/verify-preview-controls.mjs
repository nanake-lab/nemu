import {chromium} from '@playwright/test';
import {writeFile} from 'node:fs/promises';
import assert from 'node:assert/strict';
const browser=await chromium.launch({headless:true,args:['--ignore-gpu-blocklist','--use-gl=angle']});
try{
 const page=await browser.newPage({viewport:{width:1440,height:1000}});page.on('pageerror',e=>console.log('PAGEERROR',String(e)));
 await page.goto('http://127.0.0.1:5186/test/jitter/review.html');await page.waitForFunction(()=>window.__P205_READY__===true);
 const evidence=[];
 for(const preset of ['light','standard','calm']){
  await page.selectOption('#preset',preset);
  evidence.push(await page.evaluate((preset)=>({preset,version:window.__p205Review.currentVrm.meta.version,tail:[...window.__p205Review.currentVrm.springBoneManager.joints].filter(j=>j.bone.name.startsWith('NemuTail')).map(j=>({bone:j.bone.name,stiffness:j.settings.stiffness,dragForce:j.settings.dragForce,gravityPower:j.settings.gravityPower}))}),preset));
  const expected = await page.evaluate(async (preset) => (await (await fetch('/p205/data')).json()).presets[preset].tail, preset);
  for (const joint of evidence.at(-1).tail) {
   assert.equal(joint.stiffness, expected.stiffness);
   assert.equal(joint.dragForce, expected.dragForce);
   assert.equal(joint.gravityPower, expected.gravityPower);
  }
 }
 await page.selectOption('#preset','standard');
 await page.screenshot({path:'reports/P2-05/selected-A-review-controls.png'});
 await page.click('#save');await page.waitForFunction(()=>document.querySelector('#status').textContent.includes('已保存候选'));
 evidence.push({saved:await page.locator('#status').textContent()});
 const bad = await page.evaluate(async()=>{const d=await(await fetch('/p205/data')).json();d.presets.standard.hair.dragForce=2;const r=await fetch('/p205/save',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({parameters:d.presets.standard})});return {status:r.status,body:await r.json()};});
 assert.equal(bad.status,400);assert.match(bad.body.error,/hair.dragForce/);evidence.push({invalidDragRejected:bad});
 await writeFile('reports/P2-05/preview-controls-verification.json',JSON.stringify(evidence,null,2)+'\n');
}finally{await browser.close();}
