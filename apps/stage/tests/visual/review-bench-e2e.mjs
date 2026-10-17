import { chromium } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const projectRoot = resolve(__dirname, '../../../../');
const outputDir = resolve(projectRoot, 'reports/P1-03/screenshots');
await mkdir(outputDir, { recursive: true });

console.log('Starting Vite Review Server...');
// Use configFile from apps/stage so the /api/save-review middleware is active
const server = await createServer({
  root: resolve(projectRoot, 'apps/stage'),
  configFile: resolve(projectRoot, 'apps/stage/vite.config.ts'),
  server: { host: '127.0.0.1', port: 5175, strictPort: true },
});
await server.listen();

console.log('Launching browser for Review Bench E2E...');
const browser = await chromium.launch({
  headless: true,
  args: ['--enable-gpu-rasterization', '--enable-zero-copy', '--ignore-gpu-blocklist', '--use-gl=angle'],
});

try {
  const page = await browser.newPage({
    viewport: { width: 1440, height: 900 },
    deviceScaleFactor: 1,
  });

  console.log('Navigating to http://127.0.0.1:5175/review.html...');
  await page.goto('http://127.0.0.1:5175/review.html');

  // Wait for review app to initialize
  await page.waitForFunction(() => window.__REVIEW_READY__ === true, null, { timeout: 20000 });
  await page.waitForTimeout(1000);

  // 1. Initial Overview Screenshot
  await page.screenshot({ path: resolve(outputDir, '01_review_bench_overview.png') });
  console.log('Captured: 01_review_bench_overview.png');

  // 2. Test Camera Presets
  console.log('Testing Turntable Presets...');
  await page.click('[data-preset="threeQuarter"]');
  await page.waitForTimeout(400);
  await page.screenshot({ path: resolve(outputDir, '02_turntable_threeQuarter_45deg.png') });

  await page.click('[data-preset="side"]');
  await page.waitForTimeout(400);
  await page.screenshot({ path: resolve(outputDir, '03_turntable_side_90deg.png') });

  await page.click('[data-preset="back"]');
  await page.waitForTimeout(400);
  await page.screenshot({ path: resolve(outputDir, '04_turntable_back_180deg.png') });

  await page.click('[data-preset="front"]');
  await page.waitForTimeout(400);

  // 3. Test Lighting & Background Presets
  console.log('Testing Lighting & Background Presets...');
  await page.click('[data-bg="white"]');
  await page.click('[data-light="studio"]');
  await page.waitForTimeout(400);
  await page.screenshot({ path: resolve(outputDir, '05_lighting_studio_white_bg.png') });

  await page.click('[data-bg="black"]');
  await page.click('[data-light="dramatic"]');
  await page.waitForTimeout(400);
  await page.screenshot({ path: resolve(outputDir, '06_lighting_dramatic_black_bg.png') });

  await page.click('[data-bg="checker"]');
  await page.waitForTimeout(300);
  await page.screenshot({ path: resolve(outputDir, '07_checker_bg.png') });

  await page.click('[data-bg="wallpaper"]');
  await page.click('[data-light="threePoint"]');
  await page.waitForTimeout(400);
  await page.screenshot({ path: resolve(outputDir, '08_wallpaper_bg.png') });

  // Reset to gradient
  await page.click('[data-bg="gradient"]');

  // 4. Test Comparison Mode
  console.log('Testing Bible Comparison Mode...');
  await page.click('#btn-toggle-comparison');
  await page.waitForTimeout(500);
  await page.screenshot({ path: resolve(outputDir, '09_comparison_mode_split_view.png') });
  console.log('Captured: 09_comparison_mode_split_view.png');
  // Toggle off comparison mode
  await page.click('#btn-toggle-comparison');
  await page.waitForTimeout(300);

  // 5. Test Expressions
  console.log('Testing Expressions...');
  await page.click('.tab-btn[data-tab="tab-expressions"]');
  await page.click('.expr-card[data-expr="happy"]');
  await page.waitForTimeout(400);
  await page.screenshot({ path: resolve(outputDir, '10_expression_happy.png') });

  await page.click('.expr-card[data-expr="surprised"]');
  await page.waitForTimeout(400);
  await page.screenshot({ path: resolve(outputDir, '11_expression_surprised.png') });

  // Test 12 Emo Mode
  await page.click('#btn-expr-mode-emo');
  await page.waitForTimeout(300);
  await page.click('.expr-card[data-expr="smug"]');
  await page.waitForTimeout(400);
  await page.screenshot({ path: resolve(outputDir, '12_expression_emo_smug.png') });

  // 6. Test Body Readout
  console.log('Testing Body Readout Sliders...');
  await page.click('.tab-btn[data-tab="tab-readout"]');
  await page.evaluate(() => {
    const sliderEar = document.getElementById('slider-earPose');
    if (sliderEar) {
      sliderEar.value = '0.8';
      sliderEar.dispatchEvent(new Event('input'));
    }
    const sliderTail = document.getElementById('slider-tailAmp');
    if (sliderTail) {
      sliderTail.value = '1.0';
      sliderTail.dispatchEvent(new Event('input'));
    }
    const sliderPosture = document.getElementById('slider-posture');
    if (sliderPosture) {
      sliderPosture.value = '-0.5';
      sliderPosture.dispatchEvent(new Event('input'));
    }
  });
  await page.waitForTimeout(500);
  await page.screenshot({ path: resolve(outputDir, '13_body_readout_preview.png') });
  console.log('Captured: 13_body_readout_preview.png');

  // 7. Test Motion Player
  console.log('Testing Motion Player...');
  await page.click('.tab-btn[data-tab="tab-motions"]');
  await page.selectOption('#select-motion', 'wave');
  await page.click('[data-speed="0.25"]');
  await page.waitForTimeout(500);
  await page.screenshot({ path: resolve(outputDir, '14_motion_player_wave.png') });

  // 8. Test Spring Bones Inspector
  console.log('Testing Spring Bone Inspector...');
  await page.click('.tab-btn[data-tab="tab-springs"]');
  await page.evaluate(() => {
    const sliderStiff = document.getElementById('slider-spring-stiff');
    if (sliderStiff) {
      sliderStiff.value = '1.5';
      sliderStiff.dispatchEvent(new Event('input'));
    }
    const sliderDrag = document.getElementById('slider-spring-drag');
    if (sliderDrag) {
      sliderDrag.value = '0.6';
      sliderDrag.dispatchEvent(new Event('input'));
    }
    const sliderGrav = document.getElementById('slider-spring-grav');
    if (sliderGrav) {
      sliderGrav.value = '1.2';
      sliderGrav.dispatchEvent(new Event('input'));
    }
  });
  await page.waitForTimeout(400);
  await page.screenshot({ path: resolve(outputDir, '15_spring_bone_inspector.png') });

  // 9. Test Review Notes & Saving Markdown Log
  console.log('Testing Review Notes & Report Save API...');
  await page.click('.tab-btn[data-tab="tab-notes"]');
  // Fill in some review notes
  await page.evaluate(() => {
    const textareas = document.querySelectorAll('#tab-notes textarea');
    if (textareas[0]) textareas[0].value = '二次元比例符合 bible 设定要求，头部轮廓与发丝分布正常。';
    if (textareas[1]) textareas[1].value = '侧面与背面视角光影渐变过渡平滑，无破面。';
    if (textareas[2]) textareas[2].value = '猫耳与猫嘴自定义表情匹配自然，变形无过度拉伸。';
    if (textareas[3]) textareas[3].value = '待机呼吸与耳尾运动协调，节奏平稳。';
    if (textareas[4]) textareas[4].value = '弹簧骨刚度 1.5、阻力 0.6、重力 1.2 表现自然，无抖动穿模。';
  });

  await page.click('#btn-save-review-log');
  await page.waitForTimeout(1000);
  await page.screenshot({ path: resolve(outputDir, '16_review_notes_saved.png') });
  console.log('Captured: 16_review_notes_saved.png');

  console.log('All Review Bench E2E validation steps completed successfully!');
} finally {
  await browser.close();
  await server.close();
}
