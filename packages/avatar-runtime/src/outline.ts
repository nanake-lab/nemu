import type { VRM } from '@pixiv/three-vrm';
import * as THREE from 'three';

export interface OutlineAdaptOptions {
  referenceHeightPx?: number;
  minHeightPx?: number;
  maxHeightPx?: number;
}

/**
 * Adapts MToon outline width factor based on the canvas pixel height,
 * keeping the visual outline thickness consistent across the 300-900 px height range.
 */
export function adaptOutlineWidth(
  vrm: VRM,
  canvasHeightPx: number,
  options: OutlineAdaptOptions = {},
): number {
  const referenceHeight = options.referenceHeightPx ?? 600;
  const minHeight = options.minHeightPx ?? 300;
  const maxHeight = options.maxHeightPx ?? 900;

  // Clamp height to expected range
  const clampedHeight = THREE.MathUtils.clamp(canvasHeightPx, minHeight, maxHeight);
  // Scale factor: as height decreases, increase outline factor so lines don't disappear;
  // as height increases, keep lines sharp and delicate.
  const scaleRatio = referenceHeight / clampedHeight;

  let materialsModified = 0;

  vrm.scene.traverse((obj) => {
    if ((obj as THREE.Mesh).isMesh) {
      const mesh = obj as THREE.Mesh;
      const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material];

      for (const mat of materials) {
        if (!mat) continue;
        const mtoonMat = mat as unknown as {
          outlineWidthFactor?: number;
          outlineWidthMode?: string;
          userData?: Record<string, unknown>;
        };

        if (typeof mtoonMat.outlineWidthFactor === 'number' && mtoonMat.outlineWidthFactor > 0) {
          if (!mat.userData) mat.userData = {};
          if (typeof mat.userData.baseOutlineWidthFactor !== 'number') {
            mat.userData.baseOutlineWidthFactor = mtoonMat.outlineWidthFactor;
          }

          const baseFactor = mat.userData.baseOutlineWidthFactor as number;
          // Apply adapted factor
          mtoonMat.outlineWidthFactor = baseFactor * scaleRatio;
          materialsModified++;
        }
      }
    }
  });

  return materialsModified;
}
