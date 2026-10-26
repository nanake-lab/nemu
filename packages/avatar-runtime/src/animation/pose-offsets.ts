import type { Object3D, Quaternion, Vector3 } from 'three';

interface FramePose {
  node: Object3D;
  position: Vector3;
  quaternion: Quaternion;
  outputPosition: Vector3;
  outputQuaternion: Quaternion;
}

/** Removes the previous frame's offsets before L0; preserves external pose writes. */
export class FramePoseOffsets {
  private poses: FramePose[] = [];

  restore(): void {
    for (const pose of this.poses) {
      if (pose.node.position.equals(pose.outputPosition)) pose.node.position.copy(pose.position);
      if (pose.node.quaternion.equals(pose.outputQuaternion))
        pose.node.quaternion.copy(pose.quaternion);
    }
    this.poses = [];
  }

  capture(nodes: Iterable<Object3D | null>): void {
    this.poses = [...new Set(nodes)]
      .filter((node): node is Object3D => node !== null)
      .map((node) => ({
        node,
        position: node.position.clone(),
        quaternion: node.quaternion.clone(),
        outputPosition: node.position.clone(),
        outputQuaternion: node.quaternion.clone(),
      }));
  }

  finish(): void {
    for (const pose of this.poses) {
      pose.outputPosition.copy(pose.node.position);
      pose.outputQuaternion.copy(pose.node.quaternion);
    }
  }
}
