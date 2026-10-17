import type { VRM } from '@pixiv/three-vrm';

export interface SpringBoneParameters {
  stiffnessMultiplier: number;
  dragForce: number;
  gravityPower: number;
  gravityDirY: number;
}

export class SpringBoneInspector {
  private vrm: VRM | null = null;
  private params: SpringBoneParameters = {
    stiffnessMultiplier: 1.0,
    dragForce: 0.4,
    gravityPower: 1.0,
    gravityDirY: -1.0,
  };

  private originalStiffness = new Map<object, number>();

  bindVRM(vrm: VRM): void {
    this.vrm = vrm;
    this.originalStiffness.clear();

    const manager = vrm.springBoneManager;
    if (!manager) return;

    for (const joint of manager.joints) {
      if ('stiffnessForce' in joint && typeof joint.stiffnessForce === 'number') {
        this.originalStiffness.set(joint, joint.stiffnessForce);
      }
    }
    this.applyParameters();
  }

  setParameters(params: Partial<SpringBoneParameters>): void {
    this.params = { ...this.params, ...params };
    this.applyParameters();
  }

  getParameters(): SpringBoneParameters {
    return { ...this.params };
  }

  private applyParameters(): void {
    const manager = this.vrm?.springBoneManager;
    if (!manager) return;

    for (const joint of manager.joints) {
      const orig = this.originalStiffness.get(joint) ?? 1.0;
      if ('stiffnessForce' in joint) {
        (joint as { stiffnessForce: number }).stiffnessForce = orig * this.params.stiffnessMultiplier;
      }
      if ('dragForce' in joint) {
        (joint as { dragForce: number }).dragForce = this.params.dragForce;
      }
      if ('gravityPower' in joint) {
        (joint as { gravityPower: number }).gravityPower = this.params.gravityPower;
      }
      if ('gravityDir' in joint && (joint as { gravityDir?: { y: number } }).gravityDir) {
        (joint as { gravityDir: { y: number } }).gravityDir.y = this.params.gravityDirY;
      }
    }
  }

  /**
   * Resets spring bone joints (e.g. to test impulse/settling).
   */
  resetSimulation(): void {
    this.vrm?.springBoneManager?.reset();
  }

  /**
   * Exports spring bone tuning as a build.yaml snippet.
   */
  exportBuildYamlSnippet(): string {
    return `# Spring Bone Parameters for content/nemu/build.yaml
rig:
  springBones:
    stiffnessMultiplier: ${this.params.stiffnessMultiplier.toFixed(2)}
    dragForce: ${this.params.dragForce.toFixed(2)}
    gravityPower: ${this.params.gravityPower.toFixed(2)}
    gravityDir: [0.0, ${this.params.gravityDirY.toFixed(2)}, 0.0]
`;
  }
}
