import { ReviewApp } from '../../src/review/review-app.ts';
import { IdlePreview } from './idle-preview.mjs';
import { applySpringSettings } from './spring-settings.mjs';
const data = await (await fetch('/p205/data')).json();
const canvas = document.getElementById('review-canvas');
const app = new ReviewApp(canvas, document.getElementById('viewport-container'));
window.__p205Review = app;
await app.loadModel('/p205/nemu.vrm', '/p205/nemu.profile.json');
app.stop();
const vrm = app.currentVrm;
if (vrm.meta.version !== 'P2-05-candidate-short-thick-standard') throw new Error(`Unexpected avatar ${vrm.meta.version}`);
const labels = { hair: '头发', ear: '猫耳', tail: '尾巴', skirt: '裙摆' };
let parameters = structuredClone(data.presets[data.selectedSpringPreset ?? 'calm']);
let playing = false;
let black = false;
let animationId;
const idle = new IdlePreview(vrm, app.currentProfile);
const idleToggle = document.getElementById('idle');
const turnToggle = document.getElementById('slow-turn');
window.__p205Idle = idle;
const status = document.getElementById('status');
function render() { app.render(); }
function reset() {
  playing = false;
  cancelAnimationFrame(animationId);
  idleToggle.checked = false;
  idle.reset();
  render();
}
function apply() {
  const resumeIdle = idleToggle.checked;
  applySpringSettings(vrm, parameters);
  reset();
  if (resumeIdle) startIdle();
}
function startIdle() {
  reset();
  idleToggle.checked = true;
  playing = true;
  let previous = performance.now();
  function frame(now) {
    if (!playing) return;
    const dt = Math.min((now - previous) / 1000, 1 / 30);
    previous = now;
    idle.step(dt, turnToggle.checked);
    render();
    status.textContent = `日常待机 ${idle.elapsed.toFixed(1)} 秒${turnToggle.checked ? ' · 缓慢转身' : ''}；可直接切换弹簧预设。`;
    animationId = requestAnimationFrame(frame);
  }
  animationId = requestAnimationFrame(frame);
}
idleToggle.onchange = () => {
  if (idleToggle.checked) startIdle();
  else { reset(); status.textContent = '待机已关闭，已归零。'; }
};
canvas.addEventListener('pointermove', (event) => {
  const rect = canvas.getBoundingClientRect();
  idle.gaze = {
    x: Math.max(-1, Math.min(1, (event.clientX - rect.left) / rect.width * 2 - 1)),
    y: Math.max(-1, Math.min(1, 1 - (event.clientY - rect.top) / rect.height * 2)),
  };
});
canvas.addEventListener('pointerleave', () => { idle.gaze = { x: 0, y: 0 }; });
document.addEventListener('visibilitychange', () => {
  if (document.hidden) { reset(); status.textContent = '页面已隐藏，播放已停止。'; }
});
function controls() {
  const parent = document.getElementById('parameters');
  parent.replaceChildren();
  for (const [kind, label] of Object.entries(labels)) {
    const box = document.createElement('fieldset');
    const title = document.createElement('legend'); title.textContent = label; box.append(title);
    for (const [key, text, max, step] of [['stiffness', '刚度', 4, .01], ['dragForce', '阻尼', 1, .01], ['gravityPower', '重力', 1, .005]]) {
      const line = document.createElement('label'); line.append(`${text} `);
      const slider = document.createElement('input'); slider.type = 'range'; slider.min = '0'; slider.max = String(max); slider.step = String(step); slider.value = String(parameters[kind][key]);
      const value = document.createElement('output'); value.textContent = slider.value;
      slider.oninput = () => { parameters[kind][key] = Number(slider.value); value.textContent = slider.value; apply(); };
      line.append(slider, value); box.append(line);
    }
    parent.append(box);
  }
}
document.getElementById('preset').onchange = (event) => { parameters = structuredClone(data.presets[event.target.value]); controls(); apply(); status.textContent = '预设已应用到实际关节。'; };
for (const button of document.querySelectorAll('[data-view]')) button.onclick = () => { app.turntable.setPreset(button.dataset.view, false); render(); };
document.getElementById('background').onclick = () => { black = !black; app.renderer.renderer.setClearColor(black ? 0x0a0b0e : 0xffffff, 1); render(); };
document.getElementById('stop').onclick = () => { reset(); status.textContent = '已停止并归零。'; };
document.getElementById('play').onclick = () => {
  reset(); playing = true;
  const hips = vrm.humanoid.getNormalizedBoneNode('hips');
  const origin = hips.position.clone();
  const head = vrm.humanoid.getNormalizedBoneNode('head');
  const spine = vrm.humanoid.getNormalizedBoneNode('spine');
  let previous = performance.now(), elapsed = 0;
  function frame(now) {
    if (!playing) return;
    const dt = Math.min((now - previous) / 1000, 1 / 30); previous = now; elapsed += dt;
    const drive = elapsed < 30;
    hips.position.copy(origin);
    hips.position.x += drive ? .24 * Math.sin(2 * Math.PI * 1.3 * elapsed) : 0;
    hips.position.y += drive ? .12 * Math.sin(2 * Math.PI * 2.1 * elapsed) : 0;
    head.rotation.z = drive ? .18 * Math.sin(2 * Math.PI * 1.7 * elapsed) : 0;
    spine.rotation.z = drive ? .10 * Math.sin(2 * Math.PI * 1.3 * elapsed) : 0;
    vrm.humanoid.update(); vrm.scene.updateMatrixWorld(true); vrm.springBoneManager.update(dt); render();
    status.textContent = `${drive ? '驱动' : '回稳'} ${elapsed.toFixed(1)} / 35 秒；此显示不签发验收结论。`;
    if (elapsed < 35) animationId = requestAnimationFrame(frame); else { playing = false; status.textContent = '诊断播放结束，仍需检查抖动与手感。'; }
  }
  animationId = requestAnimationFrame(frame);
};
document.getElementById('save').onclick = async () => {
  const result = await fetch('/p205/save', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ parameters }) });
  const body = await result.json(); status.textContent = result.ok ? `已保存候选：${body.path}` : `保存失败：${body.error}`;
};
app.turntable.target.set(0, .80, 0); app.turntable.setDistance(5.8); app.turntable.setPreset('threeQuarter', false);
app.renderer.renderer.setClearColor(0xffffff, 1);
controls(); apply();
window.__P205_READY__ = true;
status.textContent = '候选已就绪。默认暂停，仅播放时连续渲染。';
