import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { VRMLoaderPlugin } from '@pixiv/three-vrm';
import { AvatarRenderer } from '/packages/avatar-runtime/src/renderer.ts';
const p = new URLSearchParams(location.search);
const variant = p.get('variant') || 'B';
if (!['A', 'B', 'C'].includes(variant)) throw Error('Unknown variant');
const root = '/content/_candidates/P2-06/revision2/';
const scene = new THREE.Scene();
const wrapper = new AvatarRenderer({ width: 440, height: 500, devicePixelRatio: 1 });
const renderer = wrapper.renderer;
document.body.append(renderer.domElement);
const bg = p.get('bg') || 'white';
renderer.setClearColor(bg === 'black' ? '#000000' : '#ffffff', bg === 'gradient' ? 0 : 1);
if (bg === 'gradient') document.body.style.background = 'linear-gradient(135deg,#9bc9c0,#9796b9,#dbbfce)';
scene.add(new THREE.AmbientLight(0xffffff, .45));
const key = new THREE.DirectionalLight(0xffffff, 1.1);
key.position.set(1.2, 2, 2); scene.add(key);
const rim = new THREE.DirectionalLight(0xe8f0ff, .7);
rim.position.set(-1.5, 1.8, -2); scene.add(rim);
const camera = new THREE.PerspectiveCamera(25, 440 / 500, .01, 30);
const upper = p.get('framing') === 'upperBody';
camera.position.set(0, upper ? 1.22 : 1.37, upper ? 1.65 : .72);
camera.lookAt(0, upper ? 1.22 : 1.37, 0);
const loader = new GLTFLoader();
loader.register(parser => new VRMLoaderPlugin(parser));
const old = p.get('revision') === 'r1';
const url = old ? '/content/_candidates/P2-06/models/standard.vrm' : `${root}models/${variant}.vrm`;
const gltf = await loader.loadAsync(url);
const vrm = gltf.userData.vrm;
scene.add(vrm.scene);
const diagnostic = p.get('diagnostic');
const materialInfo = [];
vrm.scene.traverse(o => {
  if (!o.isMesh) return;
  for (const m of Array.isArray(o.material) ? o.material : [o.material]) {
    if (diagnostic === 'all-outline' || diagnostic === 'mouth-outline' && m.name.includes('FaceMouth') || diagnostic === 'skin-outline' && m.name.includes('Face_00_SKIN')) {
      m.outlineWidthFactor = 0;
      m.needsUpdate = true;
    }
    if (diagnostic === 'hide-mouth' && m.name.includes('FaceMouth')) m.visible = false;
    if (diagnostic === 'no-normal' && m.name.includes('Face_00_SKIN')) { m.normalMap = null; m.needsUpdate = true; }
    if (!materialInfo.some(v => v.name === m.name)) materialInfo.push({ name: m.name, outline: m.outlineWidthFactor, map: m.map?.colorSpace });
  }
});
let bindings = [];
if (p.has('emo')) {
  const maps = await (await fetch(root + 'emotionMap.candidates.json')).json();
  bindings = maps[p.get('emo')][variant === 'C' ? 'A' : variant];
} else if (p.has('expr')) bindings = [{ expression: p.get('expr'), weight: Number(p.get('weight') || 1) }];
for (const bind of bindings) {
  if (!vrm.expressionManager.getExpression(bind.expression)) throw Error('Missing ' + bind.expression);
  vrm.expressionManager.setValue(bind.expression, bind.weight);
}
vrm.update(0);
wrapper.render(scene, camera);
document.getElementById('label').textContent = `${old ? '原版诊断' : '第二轮'} ${p.get('emo') || p.get('expr') || 'neutral'} ${variant} ${diagnostic || ''} · ${bg}`;
window.__P206_READY__ = true;
window.__P206_REPORT__ = { bindings, materialInfo, triangles: renderer.info.render.triangles, outputColorSpace: renderer.outputColorSpace, toneMapping: renderer.toneMapping, exposure: renderer.toneMappingExposure };
window.__P206_RENDER__ = () => wrapper.render(scene, camera);
