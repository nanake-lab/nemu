import * as THREE from 'three';
import type { TwoBoneIKParams, TwoBoneIKSolution } from './types.js';

/**
 * Analytical critically damped spring filter (damping ratio zeta = 1.0).
 * Guarantees zero overshoot, monotonic approach, and unconditional stability.
 */
export class CriticallyDampedSpring {
  private current: number;
  private velocity = 0;
  private omega: number;

  constructor(initialValue = 0, omega = 10.0) {
    this.current = initialValue;
    this.omega = omega;
  }

  setOmega(omega: number): void {
    this.omega = Math.max(0.001, omega);
  }

  getOmega(): number {
    return this.omega;
  }

  reset(value: number): void {
    this.current = value;
    this.velocity = 0;
  }

  getValue(): number {
    return this.current;
  }

  getVelocity(): number {
    return this.velocity;
  }

  update(target: number, deltaSec: number): number {
    if (deltaSec <= 0) return this.current;

    const delta = this.current - target;
    const exp = Math.exp(-this.omega * deltaSec);
    const temp = (this.velocity + this.omega * delta) * deltaSec;

    this.current = target + (delta + temp) * exp;
    this.velocity = (this.velocity - this.omega * temp) * exp;

    return this.current;
  }

  /**
   * Update for angular values (in radians) with wrap-around at [-PI, PI].
   */
  updateAngle(targetAngleRad: number, deltaSec: number): number {
    if (deltaSec <= 0) return this.current;

    let diff = (targetAngleRad - this.current) % (Math.PI * 2);
    if (diff > Math.PI) diff -= Math.PI * 2;
    if (diff < -Math.PI) diff += Math.PI * 2;

    const targetUnwrapped = this.current + diff;
    const delta = this.current - targetUnwrapped;
    const exp = Math.exp(-this.omega * deltaSec);
    const temp = (this.velocity + this.omega * delta) * deltaSec;

    this.current = targetUnwrapped + (delta + temp) * exp;
    this.velocity = (this.velocity - this.omega * temp) * exp;

    // Normalize to [-PI, PI]
    this.current = ((this.current + Math.PI) % (Math.PI * 2) + Math.PI * 2) % (Math.PI * 2) - Math.PI;

    return this.current;
  }
}

/**
 * Ring buffer for retrieving time-delayed values.
 */
export class DelayBuffer<T> {
  private readonly buffer: Array<{ time: number; value: T }> = [];
  private currentTime = 0;
  private readonly maxDurationSec: number;

  constructor(maxDurationSec = 2.0) {
    this.maxDurationSec = maxDurationSec;
  }

  push(value: T, deltaSec: number): void {
    this.currentTime += deltaSec;
    this.buffer.push({ time: this.currentTime, value });

    // Evict entries older than max duration
    const cutoff = this.currentTime - this.maxDurationSec;
    while (this.buffer.length > 2 && this.buffer[0]!.time < cutoff) {
      this.buffer.shift();
    }
  }

  getDelayed(delaySec: number): T | null {
    if (this.buffer.length === 0) return null;
    const targetTime = this.currentTime - delaySec;

    if (targetTime <= this.buffer[0]!.time) {
      return this.buffer[0]!.value;
    }
    const last = this.buffer[this.buffer.length - 1]!;
    if (targetTime >= last.time) {
      return last.value;
    }

    // Binary search for the closest earlier entry
    let low = 0;
    let high = this.buffer.length - 1;
    while (low <= high) {
      const mid = (low + high) >> 1;
      if (this.buffer[mid]!.time < targetTime) {
        low = mid + 1;
      } else {
        high = mid - 1;
      }
    }

    const idx = Math.max(0, Math.min(this.buffer.length - 1, low));
    return this.buffer[idx]!.value;
  }

  reset(): void {
    this.buffer.length = 0;
    this.currentTime = 0;
  }
}

/**
 * Smooth Hermite cubic interpolation: 3t^2 - 2t^3
 */
export function smoothHermite(t: number): number {
  const clamped = Math.max(0, Math.min(1, t));
  return clamped * clamped * (3 - 2 * clamped);
}

/**
 * Computes angular velocity between two quaternions in rad/s.
 */
export function computeAngularVelocity(
  q1: THREE.Quaternion,
  q2: THREE.Quaternion,
  deltaSec: number,
): number {
  if (deltaSec <= 0) return 0;
  // Dot product of normalized quaternions
  const dot = Math.abs(q1.x * q2.x + q1.y * q2.y + q1.z * q2.z + q1.w * q2.w);
  const clampedDot = Math.max(-1, Math.min(1, dot));
  const angleRad = 2 * Math.acos(clampedDot);
  return angleRad / deltaSec;
}

/**
 * Auto-aligns loop head and tail frames for an AnimationClip.
 * Ensures the rotation difference between start and end frames is <= 2 degrees (0.0349 rad).
 */
export function alignLoopHeadToTail(clip: THREE.AnimationClip): THREE.AnimationClip {
  const maxDiffRad = THREE.MathUtils.degToRad(2.0); // 2 degrees

  for (const track of clip.tracks) {
    if (track instanceof THREE.QuaternionKeyframeTrack) {
      const values = track.values;
      const count = track.times.length;
      if (count < 2) continue;

      const qStart = new THREE.Quaternion(values[0], values[1], values[2], values[3]);
      const endIdx = (count - 1) * 4;
      const qEnd = new THREE.Quaternion(
        values[endIdx],
        values[endIdx + 1],
        values[endIdx + 2],
        values[endIdx + 3],
      );

      const dot = Math.abs(qStart.dot(qEnd));
      const diffRad = 2 * Math.acos(Math.max(-1, Math.min(1, dot)));

      // If rotation discrepancy > 0, align tail frame to match head frame seamlessly
      if (diffRad > 0.0001) {
        // Ensure shortest path dot > 0
        if (qStart.dot(qEnd) < 0) {
          qEnd.set(-qEnd.x, -qEnd.y, -qEnd.z, -qEnd.w);
        }

        // Snap or blend last frame to match start frame
        values[endIdx] = qStart.x;
        values[endIdx + 1] = qStart.y;
        values[endIdx + 2] = qStart.z;
        values[endIdx + 3] = qStart.w;

        // Smoothly interpolate the last 15% of frames towards qStart to avoid abrupt jerk
        const blendCount = Math.max(1, Math.floor(count * 0.15));
        for (let i = 1; i <= blendCount; i++) {
          const idx = (count - 1 - i) * 4;
          const qCurrent = new THREE.Quaternion(
            values[idx],
            values[idx + 1],
            values[idx + 2],
            values[idx + 3],
          );
          const t = i / (blendCount + 1);
          // Interpolate towards qStart
          const factor = (1 - t) * Math.min(1, diffRad / maxDiffRad);
          qCurrent.slerp(qStart, factor);
          values[idx] = qCurrent.x;
          values[idx + 1] = qCurrent.y;
          values[idx + 2] = qCurrent.z;
          values[idx + 3] = qCurrent.w;
        }
      }
    }
  }

  return clip;
}

/**
 * Analytic Two-Bone IK Solver (law of cosines).
 */
export function solveTwoBoneIK(params: TwoBoneIKParams): TwoBoneIKSolution {
  const { rootPos, midPos, endPos, targetPos, poleVector } = params;

  const a = rootPos.distanceTo(midPos); // Upper limb length
  const b = midPos.distanceTo(endPos);  // Lower limb length
  const toTarget = new THREE.Vector3().subVectors(targetPos, rootPos);
  const targetDist = toTarget.length();

  // Clamp target distance to avoid unreachable singularities
  const epsilon = 0.0001;
  const maxReach = a + b - epsilon;
  const minReach = Math.abs(a - b) + epsilon;
  const clampedDist = Math.max(minReach, Math.min(maxReach, targetDist));

  // Law of cosines:
  // c^2 = a^2 + b^2 - 2ab * cos(beta) -> mid joint angle
  // b^2 = a^2 + c^2 - 2ac * cos(alpha) -> root angle offset
  const cosAlpha = Math.max(
    -1,
    Math.min(1, (a * a + clampedDist * clampedDist - b * b) / (2 * a * clampedDist)),
  );
  const alpha = Math.acos(cosAlpha);


  // Compute bend plane normal from pole vector
  const targetDir = toTarget.clone().normalize();
  const poleDir = new THREE.Vector3().subVectors(poleVector, rootPos).normalize();
  let bendPlaneNormal = new THREE.Vector3().crossVectors(targetDir, poleDir).normalize();
  if (bendPlaneNormal.lengthSq() < epsilon) {
    bendPlaneNormal = new THREE.Vector3(0, 0, 1);
  }

  // Direction from root to mid
  const midDir = targetDir
    .clone()
    .applyAxisAngle(bendPlaneNormal, alpha)
    .normalize();

  // Solved mid position
  const solvedMidPos = rootPos.clone().addScaledVector(midDir, a);

  // Solved rotations:
  // Root rotation aligns (midPos - rootPos) to (solvedMidPos - rootPos)
  const initialRootToMid = new THREE.Vector3().subVectors(midPos, rootPos).normalize();
  const desiredRootToMid = midDir.clone();
  const rootRotation = new THREE.Quaternion().setFromUnitVectors(initialRootToMid, desiredRootToMid);

  // Mid rotation aligns (endPos - midPos) to (targetPos - solvedMidPos)
  const initialMidToEnd = new THREE.Vector3().subVectors(endPos, midPos).normalize();
  const desiredMidToEnd = new THREE.Vector3().subVectors(targetPos, solvedMidPos).normalize();
  const midRotation = new THREE.Quaternion().setFromUnitVectors(initialMidToEnd, desiredMidToEnd);

  return { rootRotation, midRotation };
}
