// Review-only adapter for the merged P3-01 L2/L3. No production/runtime patches.
import { ProceduralLayer, FacialLayer } from '@nemu/avatar-runtime';

export class IdlePreview {
  constructor(vrm, profile) {
    this.vrm = vrm;
    this.profile = profile;
    this.gaze = { x: 0, y: 0 };
    this.elapsed = 0;
    this.turnBlend = 0;
    this.createLayers();
  }

  createLayers() {
    this.body?.dispose();
    this.face?.dispose();
    this.body = new ProceduralLayer({ vrm: this.vrm, profile: this.profile });
    // Stable blink sequence makes each preset comparison reproducible.
    let seed = 205;
    this.face = new FacialLayer({ vrm: this.vrm, profile: this.profile, prng: () => {
      seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
      return seed / 4294967296;
    } });
  }

  basePose() {
    // P3-01 L2 adds offsets: restore the base EVERY frame to prevent drift.
    this.vrm.humanoid.resetNormalizedPose();
    this.vrm.humanoid.getNormalizedBoneNode('leftUpperArm').rotation.z = -1.05;
    this.vrm.humanoid.getNormalizedBoneNode('rightUpperArm').rotation.z = 1.05;
  }

  reset() {
    this.elapsed = 0;
    this.turnBlend = 0;
    this.createLayers();
    this.basePose();
    this.vrm.expressionManager?.setValue('blink', 0);
    this.vrm.expressionManager?.update();
    this.vrm.humanoid.update();
    this.vrm.scene.updateMatrixWorld(true);
    this.vrm.springBoneManager.reset();
  }

  step(dt, turning) {
    this.elapsed += dt;
    this.basePose();
    // Optional fixture motion: +/-20 degrees, 16 s period. Ease toggle changes.
    this.turnBlend += ((turning ? 1 : 0) - this.turnBlend) * (1 - Math.exp(-dt * 3));
    this.vrm.humanoid.getNormalizedBoneNode('hips').rotation.y =
      this.turnBlend * Math.PI / 9 * Math.sin(this.elapsed * Math.PI / 8);
    this.vrm.humanoid.update();
    this.vrm.scene.updateMatrixWorld(true);
    this.body.setGazeTarget(this.gaze);
    this.body.update(dt);
    this.face.update(dt);
    // Transfer normalized animation into raw bones before spring simulation.
    this.vrm.humanoid.update();
    this.vrm.scene.updateMatrixWorld(true);
    this.vrm.springBoneManager.update(dt); // exactly one solver step per frame
    this.vrm.scene.updateMatrixWorld(true);
  }
}
