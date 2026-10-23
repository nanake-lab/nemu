import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { VRMLoaderPlugin } from '@pixiv/three-vrm';
import { AvatarRenderer } from '/packages/avatar-runtime/src/renderer.ts';
const settings = await (
  await fetch('/content/_candidates/P2-06/skin/comparison-settings.json')
).json();
const wrapper = new AvatarRenderer({ width: 440, height: 500, devicePixelRatio: 1 });
const renderer = wrapper.renderer;
document.body.append(renderer.domElement);
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(
  settings.camera.fov,
  440 / 500,
  settings.camera.near,
  settings.camera.far,
);
const loader = new GLTFLoader();
loader.register((parser) => new VRMLoaderPlugin(parser));
const models = new Map();
let vrm;
let materials = [];
let lights = [];
const textureFields = [
  'map',
  'emissiveMap',
  'shadeMultiplyTexture',
  'matcapTexture',
  'normalMap',
  'outlineWidthMultiplyTexture',
  'uvAnimationMaskTexture',
  'shadingShiftTexture',
];
async function load(which) {
  if (vrm) scene.remove(vrm.scene);
  if (!models.has(which)) {
    const url = ['A', 'B', 'C'].includes(which)
      ? `/tools/asset-pipeline/.cache/P2-06-skin-lint/skin-${which}.vrm`
      : settings[which];
    if (!url) throw Error('Unknown model ' + which);
    const gltf = await loader.loadAsync(url);
    const model = gltf.userData.vrm;
    const originals = new Map();
    model.scene.traverse((o) => {
      if (o.isMesh)
        for (const m of Array.isArray(o.material) ? o.material : [o.material]) {
          if (!originals.has(m)) originals.set(m, m.clone());
        }
    });
    models.set(which, { model, originals });
  }
  const cached = models.get(which);
  vrm = cached.model;
  scene.add(vrm.scene);
  materials = [];
  for (const [m, original] of cached.originals) {
    m.copy(original);
    materials.push(m);
  }
  vrm.update(0);
}
function report() {
  return {
    threeRevision: THREE.REVISION,
    colorManagement: THREE.ColorManagement.enabled,
    workingColorSpace: THREE.ColorManagement.workingColorSpace,
    outputColorSpace: renderer.outputColorSpace,
    drawingBufferColorSpace: renderer.getContext().drawingBufferColorSpace,
    toneMapping: renderer.toneMapping,
    exposure: renderer.toneMappingExposure,
    antialiasMode: wrapper.getAntialiasMode(),
    materials: materials.map((m) => ({
      name: m.name,
      type: m.type,
      toneMapped: m.toneMapped,
      baseColor: m.color?.toArray(),
      shadeColor: m.shadeColorFactor?.toArray(),
      rimColor: m.parametricRimColorFactor?.toArray(),
      textures: Object.fromEntries(
        textureFields
          .filter((k) => m[k])
          .map((k) => [k, { colorSpace: m[k].colorSpace, flipY: m[k].flipY, name: m[k].name }]),
      ),
    })),
  };
}
function applyRig(name) {
  for (const light of lights) scene.remove(light);
  const c = settings.lights[name];
  const key = new THREE.DirectionalLight(c.key.color, c.key.intensity);
  key.position.fromArray(c.key.position);
  const rim = new THREE.DirectionalLight(c.rim.color, c.rim.intensity);
  rim.position.fromArray(c.rim.position);
  const ambient = new THREE.AmbientLight(c.ambient.color, c.ambient.intensity);
  lights = [key, rim, ambient];
  scene.add(...lights);
}
function render() {
  wrapper.render(scene, camera);
}
window.skinHarness = {
  async show({
    model = 'model',
    light = 'stage',
    background = 'white',
    framing = 'face',
    tone = 'linear',
    variant = 'control',
    mode = 'normal',
    aa = 'defaultMSAA',
  } = {}) {
    await load(variant === 'control' ? model : variant);
    applyRig(light);
    renderer.toneMapping = tone === 'none' ? THREE.NoToneMapping : THREE.LinearToneMapping;
    renderer.toneMappingExposure = 1;
    wrapper.setAntialiasMode(aa);
    const cc = settings.camera[framing];
    camera.position.fromArray(cc.position);
    camera.lookAt(...cc.target);
    renderer.setClearColor(
      background === 'gradient' ? '#ffffff' : settings.backgrounds[background],
      background === 'gradient' ? 0 : 1,
    );
    document.body.style.background = settings.backgrounds[background];
    if (mode === 'noMatcap')
      for (const m of materials)
        if (settings.targets.includes(m.name)) m.matcapFactor.setRGB(0, 0, 0);
    if (mode === 'unlit')
      vrm.scene.traverse((o) => {
        if (o.isMesh) {
          const apply = (m) =>
            settings.targets.includes(m.name)
              ? new THREE.MeshBasicMaterial({
                  name: m.name,
                  map: m.map,
                  color: m.color,
                  side: m.side,
                  alphaTest: m.alphaTest,
                  transparent: m.transparent,
                  opacity: m.opacity,
                })
              : m;
          o.material = Array.isArray(o.material) ? o.material.map(apply) : apply(o.material);
        }
      });
    document.getElementById('label').textContent =
      `${variant} · ${light} · ${background} · ${framing} · ${mode} · ${tone}`;
    render();
    return report();
  },
  render,
  report,
  swatches() {
    const sc = new THREE.Scene();
    const cam = new THREE.OrthographicCamera(-1, 1, 1, -1, 0.1, 10);
    cam.position.z = 1;
    const vals = [
      { value: '#808080', expected: [128, 128, 128] },
      { value: [0.25, 0.25, 0.25], expected: [137, 137, 137] },
      { value: '#804020', expected: [128, 64, 32] },
    ];
    const result = [];
    for (const item of vals) {
      const color = Array.isArray(item.value)
        ? new THREE.Color().fromArray(item.value)
        : new THREE.Color(item.value);
      const mesh = new THREE.Mesh(
        new THREE.PlaneGeometry(2, 2),
        new THREE.MeshBasicMaterial({ color }),
      );
      sc.add(mesh);
      wrapper.setAntialiasMode('defaultMSAA');
      wrapper.render(sc, cam);
      const gl = renderer.getContext();
      const rgba = new Uint8Array(4);
      gl.readPixels(220, 250, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, rgba);
      result.push({ input: item.value, expected: item.expected, actual: [...rgba].slice(0, 3) });
      sc.remove(mesh);
      mesh.geometry.dispose();
      mesh.material.dispose();
    }
    return result;
  },
};
const query = new URLSearchParams(location.search);
if (query.has('variant')) await window.skinHarness.show(Object.fromEntries(query));
window.__SKIN_READY__ = true;
