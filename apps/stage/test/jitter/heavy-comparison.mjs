import { ReviewApp } from '../../src/review/review-app.ts';
import { IdlePreview } from './idle-preview.mjs';
import { applySpringSettings, springCategory } from './spring-settings.mjs';

const data = await (await fetch('/p205/data')).json();
const four = new URLSearchParams(location.search).has('four');
if (four) {
  document.documentElement.style.setProperty('--columns', '4');
  document.title = 'P2-05 四组弹簧对比';
  document.querySelector('header b').textContent = 'P2-05 · 偏粗短尾巴 · 沉稳 / B 对照 / 新 C / 新 D（均未定稿）';
  document.querySelector('a').href = '/p205/four-comparison.mp4';
}
const labels = { tail: '尾巴', ear: '猫耳', hair: '头发', skirt: '裙摆' };
const rigs = [];
const fps = 30, duration = 24, dt = 1 / 120;
let frameIndex = 0, playing = false, animationId;
const timeline = document.getElementById('timeline');
for (const candidate of four ? data.fourComparison : data.heavyComparison) {
  const panel = document.createElement('section'); panel.className = 'panel';
  const heading = document.createElement('div'); heading.className = 'heading';
  const title = document.createElement('b'); title.textContent = candidate.label;
  const note = document.createElement('small'); note.textContent = candidate.intent;
  if (four) note.textContent += ` · 诊断峰值 ${candidate.maximumHairFrameDegrees.toFixed(2)}°/帧`;
  heading.append(title, note);
  const viewport = document.createElement('div'); viewport.className = 'viewport';
  const canvas = document.createElement('canvas'); viewport.append(canvas);
  const parameters = document.createElement('div'); parameters.className = 'params';
  parameters.innerHTML = '<p>参数：K 刚度 · D 阻尼 · G 重力强度（向下）</p><table><thead><tr><th>部位</th><th>K</th><th>D</th><th>G</th></tr></thead><tbody></tbody></table>';
  for (const [part, label] of Object.entries(labels)) {
    const p = data.presets[candidate.id][part];
    const row = document.createElement('tr');
    for (const text of [label, p.stiffness.toFixed(3), p.dragForce.toFixed(3), p.gravityPower.toFixed(3)]) {
      const cell = document.createElement('td'); cell.textContent = text; row.append(cell);
    }
    parameters.querySelector('tbody').append(row);
  }
  panel.append(heading, viewport, parameters); document.getElementById('panels').append(panel);
  const app = new ReviewApp(canvas, viewport);
  await app.loadModel(four ? '/p205/four.vrm' : '/p205/nemu.vrm', four ? '/p205/four.profile.json' : '/p205/nemu.profile.json'); app.stop();
  if (!app.currentVrm || app.currentVrm.meta.version !== 'P2-05-candidate-short-thick-standard') throw new Error('Unexpected comparison model');
  const counts = applySpringSettings(app.currentVrm, data.presets[candidate.id]);
  const idle = new IdlePreview(app.currentVrm, app.currentProfile);
  app.turntable.target.set(0, .80, 0); app.turntable.setDistance(5.7); app.turntable.setPreset('threeQuarter', false);
  app.renderer.renderer.setClearColor(0xffffff, 1);
  rigs.push({ app, idle, candidate, counts });
}

function driveAt(t) {
  // Identical body motion in all three columns. No animated offsets on spring bones.
  let yaw = 0, x = 0, y = 0, headRoll = 0;
  if (t >= 4 && t < 12) {
    const u = (t - 4) / 8;
    yaw = Math.PI / 9 * Math.sin(2 * Math.PI * u) * Math.sin(Math.PI * u) ** 2;
  }
  if (t >= 12 && t < 15) {
    const u = t - 12, envelope = Math.sin(Math.PI * u / 3) ** 2;
    x = .18 * Math.sin(2 * Math.PI * 1.8 * u) * envelope;
    y = .055 * Math.sin(2 * Math.PI * 2.2 * u) * envelope;
    headRoll = .12 * Math.sin(2 * Math.PI * 1.8 * u) * envelope;
  }
  return { yaw, x, y, headRoll };
}
function phase(t) {
  if (t < 4) return '① 待机呼吸 · 0–4 秒';
  if (t < 12) return '② 缓慢转身 · 4–12 秒';
  if (t < 15) return '③ 快速拖动 · 12–15 秒';
  return '④ 停止拖动后回稳 · 15–24 秒（呼吸继续）';
}
function render() {
  for (const { app } of rigs) app.render();
  timeline.textContent = `${phase(frameIndex / fps)}　｜　${(frameIndex / fps).toFixed(2)} / ${duration} 秒　｜　${four ? '四' : '三'}列同动作、原速、仅弹簧参数不同`;
}
function pause() { playing = false; cancelAnimationFrame(animationId); }
function reset() {
  pause(); frameIndex = 0;
  for (const { idle } of rigs) {
    idle.reset();
    // Common four-second warm-up avoids comparing different initial gravity transients.
    for (let i = 0; i < 480; i++) idle.step(dt, false);
    idle.elapsed = 0; idle.createLayers();
  }
  render();
}
function stepToFrame(target) {
  if (target < frameIndex || target > duration * fps) throw new Error('Comparison frame out of order');
  while (frameIndex < target) {
    for (let substep = 1; substep <= 4; substep++) {
      const t = (frameIndex + substep / 4) / fps;
      const drive = driveAt(t);
      for (const { idle } of rigs) idle.step(dt, false, drive);
    }
    frameIndex++;
  }
  render();
}
function sample() {
  return rigs.map(({ app, candidate }) => {
    const vrm = app.currentVrm;
    const joints = [...vrm.springBoneManager.joints];
    const parts = {};
    for (const part of Object.keys(labels)) {
      const region = joints.filter(j => springCategory(j.bone.name) === part);
      const joint = region.at(-1);
      const world = joint.bone.getWorldPosition(joint.bone.position.clone());
      parts[part] = { tip: world.toArray(), rotations: region.map(j => j.bone.quaternion.toArray()) };
    }
    return { id: candidate.id, parts };
  });
}
document.getElementById('play').onclick = () => {
  if (playing) return;
  if (frameIndex === duration * fps) reset();
  playing = true;
  const start = performance.now(), offset = frameIndex;
  function tick(now) {
    if (!playing) return;
    stepToFrame(Math.min(duration * fps, offset + Math.floor((now - start) / 1000 * fps)));
    if (frameIndex < duration * fps) animationId = requestAnimationFrame(tick); else pause();
  }
  animationId = requestAnimationFrame(tick);
};
document.getElementById('pause').onclick = pause;
document.getElementById('reset').onclick = reset;
document.addEventListener('visibilitychange', () => { if (document.hidden) pause(); });
reset();
window.__heavyCompare = { reset, stepToFrame, sample, rigs, fps, duration,
  setBackground(black) { for (const { app } of rigs) app.renderer.renderer.setClearColor(black ? 0x0a0b0e : 0xffffff, 1); render(); },
};
window.__HEAVY_READY__ = true;
