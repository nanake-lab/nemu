import { AnimationClip, QuaternionKeyframeTrack, Quaternion, Euler } from 'three';
import type { VRM } from '@pixiv/three-vrm';
import type { AnimationCompositor } from '@nemu/avatar-runtime';

/** Capture-only neutral arm pose; not a selected/redistributable motion-library asset. */
export function installReadoutIdleFixture(vrm: VRM, compositor: AnimationCompositor): void {
  const tracks: QuaternionKeyframeTrack[] = [];
  for (const [bone, z] of [['leftUpperArm', -1.15], ['rightUpperArm', 1.15]] as const) {
    const node = vrm.humanoid.getNormalizedBoneNode(bone);
    if (!node) continue;
    const q = new Quaternion().setFromEuler(new Euler(0, 0, z));
    tracks.push(new QuaternionKeyframeTrack(`${node.name}.quaternion`, [0, 1], [...q.toArray(), ...q.toArray()]));
  }
  compositor.l0.playClip(new AnimationClip('p306-capture-neutral-arms-candidate', 1, tracks));
}
