import {
  bodyReadoutMappingSchema,
  type BodyReadoutV2,
  type BodyReadoutMapping,
} from '@nemu/contracts';
import type { ReviewApp } from './review-app.js';

/** P3-06 controls are isolated to reduce conflicts with P2-06's shared review files. */
export function mountReadoutControls(app: ReviewApp): void {
  const root = document.getElementById('body-readout-v2-controls');
  if (!root) return;
  const title = document.createElement('h3');
  title.textContent = '身体读出 v2';
  root.append(title);
  const inputs = new Map<string, HTMLInputElement>();
  const labels = new Map<string, HTMLElement>();
  const fields: Array<{
    key: 'facing' | 'tension' | 'energy';
    label: string;
    min: number;
    neutral: number;
  }> = [
    { key: 'facing', label: '朝向你 facing', min: 0, neutral: 0.5 },
    { key: 'tension', label: '放松 / 紧绷 tension', min: -1, neutral: 0 },
    { key: 'energy', label: '精力 energy', min: 0, neutral: 0.5 },
  ];
  for (const field of fields) {
    const group = document.createElement('div');
    group.className = 'slider-group';
    const label = document.createElement('label');
    label.className = 'slider-label';
    label.htmlFor = `slider-${field.key}`;
    label.textContent = field.label;
    const value = document.createElement('span');
    value.id = `val-${field.key}`;
    value.textContent = field.neutral.toFixed(2);
    label.append(value);
    const slider = document.createElement('input');
    slider.type = 'range';
    slider.id = label.htmlFor;
    slider.min = String(field.min);
    slider.max = '1';
    slider.step = '0.01';
    slider.value = String(field.neutral);
    slider.addEventListener('input', () => {
      value.textContent = Number(slider.value).toFixed(2);
      app.applyBodyReadout({ [field.key]: Number(slider.value) });
    });
    group.append(label, slider);
    root.append(group);
    inputs.set(field.key, slider);
    labels.set(field.key, value);
  }
  const reset = document.createElement('button');
  reset.id = 'btn-reset-readout';
  reset.className = 'action-btn secondary';
  reset.textContent = '重置为中性读出';
  reset.addEventListener('click', () => {
    const neutral: Partial<BodyReadoutV2> = {};
    for (const field of fields) {
      inputs.get(field.key)!.value = String(field.neutral);
      labels.get(field.key)!.textContent = field.neutral.toFixed(2);
      neutral[field.key] = field.neutral;
    }
    app.applyBodyReadout(neutral);
  });
  root.append(reset);

  const details = document.createElement('details');
  const summary = document.createElement('summary');
  summary.textContent = '动画映射候选初值（待校准）';
  const editor = document.createElement('textarea');
  editor.id = 'readout-mapping-candidate';
  editor.rows = 13;
  editor.setAttribute('aria-label', '映射候选 JSON，最终数值由用户或 P5-17 确定');
  editor.value = JSON.stringify(app.compositor.getReadoutMapping(), null, 2);
  const status = document.createElement('p');
  status.id = 'readout-mapping-status';
  status.textContent = '仅在本次预览中应用；未保存、未定稿。';
  const apply = document.createElement('button');
  apply.id = 'btn-apply-readout-mapping';
  apply.textContent = '应用候选到预览';
  apply.addEventListener('click', () => {
    try {
      const mapping: BodyReadoutMapping = bodyReadoutMappingSchema.parse(JSON.parse(editor.value));
      app.setBodyReadoutMapping(mapping);
      status.textContent = '已应用候选；待用户或 P5-17 校准。未写入内容包。';
    } catch {
      status.textContent = '候选格式、范围或版本无效，未应用。';
    }
  });
  details.append(summary, editor, apply, status);
  root.append(details);
}
