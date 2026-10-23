import { readFileSync } from 'node:fs';
import vm from 'node:vm';
import { describe, expect, it } from 'vitest';

const template = readFileSync(new URL('../../web/annotation/index.html', import.meta.url), 'utf8');
const code = template.match(/<script type="module">\s*([\s\S]*?)<\/script>/)?.[1];
if (!code) throw new Error('Missing annotation script');

function harness() {
  const wav = Buffer.alloc(44 + 32000 * 2);
  wav.write('RIFF');
  wav.writeUInt32LE(wav.length - 8, 4);
  wav.write('WAVEfmt ', 8);
  wav.writeUInt32LE(16, 16);
  wav.writeUInt16LE(1, 20);
  wav.writeUInt16LE(1, 22);
  wav.writeUInt32LE(16000, 24);
  wav.writeUInt32LE(32000, 28);
  wav.writeUInt16LE(2, 32);
  wav.writeUInt16LE(16, 34);
  wav.write('data', 36);
  wav.writeUInt32LE(64000, 40);
  const payload = {
    method: 'synthetic-plan',
    planSha256: 'a'.repeat(64),
    sampleRate: 16000,
    tasks: Array.from({ length: 26 }, (_, i) => ({
      id: `synthetic-${i}`,
      lang: 'en',
      sha256: 'b'.repeat(64),
      frameCount: 32000,
      sampleRate: 16000,
      transcript: 'synthetic',
      wav: wav.toString('base64'),
      spectrum: '',
    })),
  };
  const context = vm.createContext({ atob, Blob, console, Uint8Array, DataView, Int16Array });
  vm.runInContext(
    `
    globalThis.saved = null; globalThis.rangePlayback = null;
    const noop=()=>{}; const drawing=new Proxy({}, {get:()=>noop,set:()=>true});
    const elements=new Map(); const listeners=new Map();
    function element(id){if(!elements.has(id))elements.set(id,{
      textContent:'',value:'',options:[],classList:{add:noop,remove:noop},disabled:false,
      clientWidth:1000,clientHeight:150,paused:true,files:[],currentTime:0,
      append(o){this.options.push(o)},getContext(){return drawing},
      addEventListener:noop,focus:noop,click(){return this.onclick?.()},
      pause(){this.paused=true},play(){this.paused=false;return Promise.resolve()}
    });return elements.get(id)}
    element('payload').textContent=${JSON.stringify(JSON.stringify(payload))};
    element('speed').value='1';
    globalThis.document={getElementById:element,createElement:()=>({})};
    globalThis.devicePixelRatio=1;
    globalThis.window={addEventListener:(type,fn)=>listeners.set(type,fn),
      showSaveFilePicker:async()=>({name:'synthetic.json',createWritable:async()=>({
        write:async value=>{saved=JSON.parse(value)},close:async()=>{}})})};
    globalThis.URL={createObjectURL:()=> 'blob:synthetic',revokeObjectURL:noop};
    globalThis.Image=class{complete=true;naturalWidth=1000;naturalHeight=257};
    globalThis.requestAnimationFrame=()=>1;globalThis.cancelAnimationFrame=noop;
    globalThis.AudioContext=class{currentTime=0;destination={};resume(){return Promise.resolve()}
      createBuffer(c,n,r){return {getChannelData:()=>new Float32Array(n)}}
      createBufferSource(){return {playbackRate:{value:1},connect:noop,stop:noop,
        start(...args){rangePlayback=args},onended:null}}};
  `,
    context,
  );
  vm.runInContext(code!, context);
  return <T>(source: string): T => vm.runInContext(source, context) as T;
}

describe('offline blind annotation logic with synthetic media (no browser)', () => {
  it('starts all 26 tasks empty and keeps the page offline', () => {
    const run = harness();
    expect(run<number>('complete()')).toBe(0);
    expect(run<unknown>('label().startSample')).toBeNull();
    expect(template).toContain("connect-src 'none'");
    expect(template).not.toMatch(/\b(fetch|XMLHttpRequest|WebSocket)\s*\(/);
  });
  it('retains labels across navigation and supports undo without automatic points', () => {
    const run = harness();
    run("mode='start';setPoint(.125);mode='end';setPoint(.75);load(1);load(0)");
    expect(run<number>('label().startSample')).toBe(2000);
    expect(run<number>('label().endSample')).toBe(12000);
    run("mode='end';setPoint(.9);$('undo').onclick()");
    expect(run<number>('label().endSample')).toBe(12000);
  });
  it('exports incomplete work as draft and complete work as final', async () => {
    const run = harness();
    await run<Promise<void>>("$('export').onclick()");
    expect(run<string>('saved.status')).toBe('draft');
    run('for(const l of labels.values()){l.startSample=100;l.endSample=200}');
    await run<Promise<void>>("$('export').onclick()");
    expect(run<string>('saved.status')).toBe('final');
    expect(run<number>('saved.samples.length')).toBe(26);
  });
  it('rejects tampered imports atomically, preserving existing labels', async () => {
    const run = harness();
    run("mode='start';setPoint(.125)");
    await run<Promise<void>>("$('export').onclick()");
    run(
      "saved.samples[0].startSample=999; saved.samples[25].sha256='tampered'; $('importFile').files=[{name:'bad.json',text:async()=>JSON.stringify(saved)}]",
    );
    await run<Promise<void>>("$('importFile').onchange()");
    expect(run<number>('label().startSample')).toBe(2000);
    expect(run<string>("$('status').textContent")).toContain('原标注保留');
  });
  it('restores a valid partial draft and uses exact audio range duration', async () => {
    const run = harness();
    run("mode='start';setPoint(.125);mode='end';setPoint(.75)");
    await run<Promise<void>>("$('export').onclick()");
    run(
      "$('clear').onclick();$('importFile').files=[{name:'draft.json',text:async()=>JSON.stringify(saved)}]",
    );
    await run<Promise<void>>("$('importFile').onchange()");
    expect(run<number>('label().endSample')).toBe(12000);
    await run<Promise<void>>('play(.125,.75)');
    expect(run<number[]>('rangePlayback')).toEqual([0, 0.125, 0.625]);
  });
  it('never falls back to automatic browser downloads when the picker is unavailable', async () => {
    const run = harness();
    run('window.showSaveFilePicker=undefined');
    await run<Promise<void>>("$('export').onclick()");
    expect(run<unknown>('saved')).toBeNull();
    expect(run<string>("$('status').textContent")).toContain('不会自动下载到 C 盘');
  });
});
