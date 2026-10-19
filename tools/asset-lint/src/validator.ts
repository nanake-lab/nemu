import type { AssetManifestEntry, NemuProfile } from '@nemu/contracts';
import type { Document } from '@gltf-transform/core';
import { resolve, dirname, basename, relative } from 'node:path';
import { readFile, stat } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
import type { JsonObject } from './glb.js';
import { readVrm } from './glb.js';

export type Severity = 'error' | 'warning';

export interface Diagnostic {
  rule: string;
  severity: Severity;
  message: string;
  repair: string;
}

export interface LintResult {
  file: string;
  errors: number;
  warnings: number;
  diagnostics: Diagnostic[];
  metrics?: LintMetrics;
}

export interface LintMetrics {
  nodeCount: number;
  meshCount: number;
  triangleCount: number;
  textureCount: number;
  imageCount: number;
  materialCount: number;
  estimatedTextureBytes: number;
  skinJointCount: number;
  maximumMeshMorphTargetCount: number;
  accessorBoundsHeightM?: number;
  fileBytes: number;
}

export interface LintOptions {
  file: string;
  profile?: string;
  manifest?: string;
}

type JsonArray = unknown[];
type Matrix = number[];

const REQUIRED_HUMAN_BONES = [
  'hips', 'spine', 'head', 'leftUpperLeg', 'leftLowerLeg', 'leftFoot', 'rightUpperLeg',
  'rightLowerLeg', 'rightFoot', 'leftUpperArm', 'leftLowerArm', 'leftHand',
  'rightUpperArm', 'rightLowerArm', 'rightHand', 'chest', 'upperChest', 'neck',
  'leftShoulder', 'rightShoulder', 'leftToes', 'rightToes',
  ...['left', 'right'].flatMap((side) => ['Thumb', 'Index', 'Middle', 'Ring', 'Little'].flatMap((finger) => {
    const start = finger === 'Thumb' ? ['Metacarpal', 'Proximal', 'Distal'] : ['Proximal', 'Intermediate', 'Distal'];
    return start.map((part) => `${side}${finger}${part}`);
  })),
];

const REQUIRED_PRESET_EXPRESSIONS = [
  'neutral', 'happy', 'angry', 'sad', 'relaxed', 'surprised', 'aa', 'ih', 'ou', 'ee', 'oh',
  'blink', 'blinkLeft', 'blinkRight',
];
const CUSTOM_EXPRESSIONS = [
  'nemuSmug', 'nemuCatMouth', 'nemuSparkle', 'nemuBlush', 'nemuTeary', 'nemuSleepy',
  'nemuPout', 'nemuAngryMark',
];
const COLLIDER_CAPSULE_BONES = [
  'neck', 'upperChest', 'leftUpperArm', 'rightUpperArm', 'leftLowerArm', 'rightLowerArm',
  'hips', 'leftUpperLeg', 'rightUpperLeg',
];
const MTOON = 'VRMC_materials_mtoon';
const UNLIT = 'KHR_materials_unlit';
const MIP_FACTOR = 4 / 3;
const MAX_TRIANGLES = 100_000;
const TARGET_TRIANGLES = 70_000;
const MAX_TEXTURE_EDGE = 2_048;
const MAX_TEXTURE_BYTES = 256 * 1024 * 1024;
const MAX_TEXTURE_COUNT = 16;
const MAX_MATERIALS = 12;
const MAX_FILE_BYTES = 60 * 1024 * 1024;
const MAX_BONES = 250;
const MAX_FACE_MORPHS = 120;
const FLOOR_FLOAT_EPSILON = 1e-6;

function object(value: unknown): JsonObject | undefined {
  return typeof value === 'object' && value !== null && !Array.isArray(value) ? value as JsonObject : undefined;
}

function array(value: unknown): JsonArray {
  return Array.isArray(value) ? value : [];
}

function string(value: unknown): string | undefined {
  return typeof value === 'string' ? value : undefined;
}

function number(value: unknown): number | undefined {
  return typeof value === 'number' && Number.isFinite(value) ? value : undefined;
}

function vector(value: unknown, length: number): number[] | undefined {
  if (!Array.isArray(value) || value.length !== length) return undefined;
  const values = value.map(number);
  return values.some((item) => item === undefined) ? undefined : values as number[];
}

function add(diagnostics: Diagnostic[], rule: string, severity: Severity, message: string, repair: string): void {
  diagnostics.push({ rule, severity, message, repair });
}

function getExtensions(value: unknown): JsonObject {
  return object(object(value)?.extensions) ?? {};
}

function multiplyMatrices(left: Matrix, right: Matrix): Matrix {
  const out = Array<number>(16).fill(0);
  const at = (matrix: Matrix, index: number): number => matrix[index] ?? 0;
  for (let column = 0; column < 4; column += 1) {
    for (let row = 0; row < 4; row += 1) {
      out[column * 4 + row] = at(left, row) * at(right, column * 4)
        + at(left, 4 + row) * at(right, column * 4 + 1)
        + at(left, 8 + row) * at(right, column * 4 + 2)
        + at(left, 12 + row) * at(right, column * 4 + 3);
    }
  }
  return out;
}

function localMatrix(node: JsonObject): Matrix {
  const matrix = vector(node.matrix, 16);
  if (matrix) return matrix;
  const translation = vector(node.translation, 3) ?? [0, 0, 0];
  const rotation = vector(node.rotation, 4) ?? [0, 0, 0, 1];
  const scale = vector(node.scale, 3) ?? [1, 1, 1];
  const x = rotation[0] ?? 0;
  const y = rotation[1] ?? 0;
  const z = rotation[2] ?? 0;
  const w = rotation[3] ?? 1;
  const x2 = x * x, y2 = y * y, z2 = z * z;
  const xy = x * y, xz = x * z, yz = y * z;
  const wx = w * x, wy = w * y, wz = w * z;
  return [
    (1 - 2 * (y2 + z2)) * (scale[0] ?? 1), (2 * (xy + wz)) * (scale[0] ?? 1), (2 * (xz - wy)) * (scale[0] ?? 1), 0,
    (2 * (xy - wz)) * (scale[1] ?? 1), (1 - 2 * (x2 + z2)) * (scale[1] ?? 1), (2 * (yz + wx)) * (scale[1] ?? 1), 0,
    (2 * (xz + wy)) * (scale[2] ?? 1), (2 * (yz - wx)) * (scale[2] ?? 1), (1 - 2 * (x2 + y2)) * (scale[2] ?? 1), 0,
    translation[0] ?? 0, translation[1] ?? 0, translation[2] ?? 0, 1,
  ];
}

function nodeWorldMatrices(nodes: JsonArray): Matrix[] {
  const parents = new Map<number, number>();
  nodes.forEach((entry, parentIndex) => array(object(entry)?.children).forEach((child) => {
    if (typeof child === 'number' && !parents.has(child)) parents.set(child, parentIndex);
  }));
  const cache = new Map<number, Matrix>();
  const resolveWorld = (index: number, visiting: Set<number>): Matrix => {
    const cached = cache.get(index);
    if (cached) return cached;
    if (visiting.has(index)) return localMatrix(object(nodes[index]) ?? {});
    const next = new Set(visiting).add(index);
    const local = localMatrix(object(nodes[index]) ?? {});
    const parent = parents.get(index);
    const world = parent === undefined ? local : multiplyMatrices(resolveWorld(parent, next), local);
    cache.set(index, world);
    return world;
  };
  return nodes.map((_, index) => resolveWorld(index, new Set()));
}

function rootAncestorIndex(nodes: JsonArray, nodeIndex: number): number {
  const parents = new Map<number, number>();
  nodes.forEach((entry, parentIndex) => array(object(entry)?.children).forEach((child) => {
    if (typeof child === 'number' && !parents.has(child)) parents.set(child, parentIndex);
  }));
  let root = nodeIndex;
  const visited = new Set<number>();
  while (parents.has(root) && !visited.has(root)) {
    visited.add(root);
    root = parents.get(root) ?? root;
  }
  return root;
}

function transformPoint(matrix: Matrix, point: number[]): number[] {
  const x = point[0] ?? 0;
  const y = point[1] ?? 0;
  const z = point[2] ?? 0;
  const at = (index: number): number => matrix[index] ?? 0;
  const w = at(3) * x + at(7) * y + at(11) * z + at(15);
  const divisor = w === 0 ? 1 : w;
  return [
    (at(0) * x + at(4) * y + at(8) * z + at(12)) / divisor,
    (at(1) * x + at(5) * y + at(9) * z + at(13)) / divisor,
    (at(2) * x + at(6) * y + at(10) * z + at(14)) / divisor,
  ];
}

function geometryBounds(json: JsonObject): { minY: number; maxY: number } | undefined {
  const nodes = array(json.nodes);
  const accessors = array(json.accessors);
  const meshes = array(json.meshes);
  const worlds = nodeWorldMatrices(nodes);
  let minY = Number.POSITIVE_INFINITY;
  let maxY = Number.NEGATIVE_INFINITY;
  nodes.forEach((nodeValue, nodeIndex) => {
    const node = object(nodeValue);
    if (typeof node?.mesh !== 'number') return;
    const mesh = object(meshes[node.mesh]);
    array(mesh?.primitives).forEach((primitiveValue) => {
      const attributes = object(object(primitiveValue)?.attributes);
      const accessor = object(accessors[Number(attributes?.POSITION)]);
      const min = vector(accessor?.min, 3);
      const max = vector(accessor?.max, 3);
      if (!min || !max) return;
      const world = worlds[nodeIndex];
      if (!world) return;
      for (const x of [min[0], max[0]]) {
        for (const y of [min[1], max[1]]) {
          for (const z of [min[2], max[2]]) {
            const transformed = transformPoint(world, [x ?? 0, y ?? 0, z ?? 0]);
            minY = Math.min(minY, transformed[1] ?? 0);
            maxY = Math.max(maxY, transformed[1] ?? 0);
          }
        }
      }
    });
  });
  return Number.isFinite(minY) && Number.isFinite(maxY) ? { minY, maxY } : undefined;
}

function faceMorphCount(json: JsonObject, vrm: JsonObject): number {
  const nodes = array(json.nodes);
  const meshes = array(json.meshes);
  const faceMeshes = new Set<number>();
  const expressions = object(vrm.expressions) ?? {};
  for (const category of ['preset', 'custom'] as const) {
    Object.values(object(expressions[category]) ?? {}).forEach((expressionValue) => {
      array(object(expressionValue)?.morphTargetBinds).forEach((bindingValue) => {
        const nodeIndex = number(object(bindingValue)?.node);
        const meshIndex = nodeIndex === undefined ? undefined : number(object(nodes[nodeIndex])?.mesh);
        if (meshIndex !== undefined) faceMeshes.add(meshIndex);
      });
    });
  }
  if (faceMeshes.size === 0) {
    nodes.forEach((nodeValue) => {
      const node = object(nodeValue);
      const name = string(node?.name) ?? '';
      const meshIndex = number(node?.mesh);
      if (meshIndex !== undefined && /face|head|顔/u.test(name)) faceMeshes.add(meshIndex);
    });
  }
  return [...faceMeshes].reduce((max, meshIndex) => {
    const mesh = object(meshes[meshIndex]);
    const targetCount = Math.max(0, ...array(mesh?.primitives).map((primitive) => array(object(primitive)?.targets).length));
    return Math.max(max, targetCount);
  }, 0);
}

function humanBoneNodes(vrm: JsonObject): Map<string, number> {
  const bones = object(object(vrm.humanoid)?.humanBones) ?? {};
  const result = new Map<string, number>();
  Object.entries(bones).forEach(([name, value]) => {
    const node = number(object(value)?.node);
    if (node !== undefined) result.set(name, node);
  });
  return result;
}

function nodeNames(json: JsonObject): string[] {
  return array(json.nodes).map((value) => string(object(value)?.name) ?? '');
}

function dimensions(image: Uint8Array): { width: number; height: number } | undefined {
  const bytes = Buffer.from(image);
  if (bytes.length >= 24 && bytes.subarray(0, 8).equals(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]))) {
    return { width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) };
  }
  if (bytes.length >= 12 && bytes.toString('ascii', 0, 4) === 'RIFF' && bytes.toString('ascii', 8, 12) === 'WEBP') {
    if (bytes.toString('ascii', 12, 16) === 'VP8X' && bytes.length >= 30) {
      const width = 1 + bytes.readUIntLE(24, 3);
      const height = 1 + bytes.readUIntLE(27, 3);
      return { width, height };
    }
  }
  const ktx2Identifier = Buffer.from([0xab, 0x4b, 0x54, 0x58, 0x20, 0x32, 0x30, 0xbb, 0x0d, 0x0a, 0x1a, 0x0a]);
  if (bytes.length >= 30 && bytes.subarray(0, 12).equals(ktx2Identifier)) {
    return { width: bytes.readUInt32LE(20), height: bytes.readUInt32LE(24) };
  }
  if (bytes.length > 4 && bytes[0] === 0xff && bytes[1] === 0xd8) {
    let offset = 2;
    while (offset + 4 <= bytes.length) {
      if (bytes[offset] !== 0xff) { offset += 1; continue; }
      const marker = bytes[offset + 1] ?? -1;
      offset += 2;
      if (marker === 0xd9 || marker === 0xda) break;
      if (offset + 2 > bytes.length) break;
      const segmentLength = bytes.readUInt16BE(offset);
      if (segmentLength < 2 || offset + segmentLength > bytes.length) break;
      const startOfFrame = [0xc0, 0xc1, 0xc2, 0xc3, 0xc5, 0xc6, 0xc7, 0xc9, 0xca, 0xcb, 0xcd, 0xce, 0xcf].includes(marker);
      if (startOfFrame && segmentLength >= 7) {
        return { height: bytes.readUInt16BE(offset + 3), width: bytes.readUInt16BE(offset + 5) };
      }
      offset += segmentLength;
    }
  }
  return undefined;
}

async function imageBytes(imageValue: unknown, json: JsonObject, binaryChunks: Uint8Array[], modelDir: string): Promise<Uint8Array | undefined> {
  const image = object(imageValue);
  if (!image) return undefined;
  const uri = string(image.uri);
  if (uri?.startsWith('data:')) {
    const comma = uri.indexOf(',');
    if (comma < 0) return undefined;
    const data = uri.slice(comma + 1);
    return uri.slice(0, comma).endsWith(';base64') ? Buffer.from(data, 'base64') : Buffer.from(decodeURIComponent(data));
  }
  if (uri) return readFile(resolve(modelDir, uri)).catch(() => undefined);
  const viewIndex = number(image.bufferView);
  if (viewIndex === undefined) return undefined;
  const view = object(array(json.bufferViews)[viewIndex]);
  const offset = number(view?.byteOffset) ?? 0;
  const length = number(view?.byteLength);
  const chunkIndex = number(view?.buffer) ?? 0;
  const bytes = binaryChunks[chunkIndex];
  if (!bytes || length === undefined || offset + length > bytes.byteLength) return undefined;
  return bytes.subarray(offset, offset + length);
}

function triangleCount(document: Document): number {
  let total = 0;
  document.getRoot().listMeshes().forEach((mesh) => mesh.listPrimitives().forEach((primitive) => {
    const count = primitive.getIndices()?.getCount() ?? primitive.getAttribute('POSITION')?.getCount() ?? 0;
    const mode = primitive.getMode() ?? 4;
    if (mode === 4) total += Math.floor(count / 3);
    else if (mode === 5 || mode === 6) total += Math.max(0, count - 2);
  }));
  return total;
}

function findExpressionObject(expressions: JsonObject, category: 'preset' | 'custom'): JsonObject {
  return object(expressions[category]) ?? {};
}

function parseSpringNodes(springValue: unknown): number[] {
  return array(object(springValue)?.joints).map((joint) => number(object(joint)?.node)).filter((node): node is number => node !== undefined);
}

function containsSequence(haystack: number[], needle: number[]): boolean {
  if (needle.length === 0) return false;
  return haystack.some((_, start) => needle.every((value, offset) => haystack[start + offset] === value));
}

function springNodeArrays(springs: JsonArray): number[][] {
  return springs.map(parseSpringNodes);
}

function nameMatch(name: string, category: 'ear' | 'tail' | 'hair' | 'skirt'): boolean {
  const normalized = name.toLowerCase();
  const patterns: Record<typeof category, RegExp> = {
    ear: /ear|mimi|cat.?ear|耳/u,
    tail: /tail|shippo|尾/u,
    hair: /hair|髪|kami/u,
    skirt: /skirt|dress|hem|スカート|スカート/u,
  };
  return patterns[category].test(normalized);
}

function validateSprings(
  diagnostics: Diagnostic[],
  json: JsonObject,
  vrm: JsonObject,
  profile: NemuProfile | undefined,
): void {
  const springExtension = object(getExtensions(json).VRMC_springBone) ?? {};
  const springs = array(springExtension.springs);
  const colliders = array(springExtension.colliders);
  const colliderGroups = array(springExtension.colliderGroups);
  const names = nodeNames(json);
  const jointArrays = springNodeArrays(springs);
  const humanBones = humanBoneNodes(vrm);
  const roleSpringIndexes = new Set<number>();
  const nodeIndexForProfileBone = (bone: string): number => {
    const byName = names.indexOf(bone);
    return byName >= 0 ? byName : humanBones.get(bone) ?? -1;
  };
  const identifySpringIndexes = (category: 'ear' | 'tail' | 'hair' | 'skirt'): number[] => springs.flatMap((spring, index) => {
    const springName = string(object(spring)?.name) ?? '';
    return nameMatch(springName, category) || (jointArrays[index] ?? []).some((node) => nameMatch(names[node] ?? '', category)) ? [index] : [];
  });

  if (profile) {
    const left = profile.rig.earBonesL.map(nodeIndexForProfileBone);
    const right = profile.rig.earBonesR.map(nodeIndexForProfileBone);
    const tail = profile.rig.tailBones.map(nodeIndexForProfileBone);
    const leftIndex = jointArrays.findIndex((nodes) => nodes.length >= 2 && containsSequence(nodes, left));
    const rightIndex = jointArrays.findIndex((nodes) => nodes.length >= 2 && containsSequence(nodes, right));
    const tailIndex = jointArrays.findIndex((nodes) => nodes.length >= 8 && containsSequence(nodes, tail));
    if (left.length < 2 || left.some((node) => node < 0) || leftIndex < 0) {
      add(diagnostics, 'A1.4-cat-ear-spring-bones', 'error', 'Left cat ear must be represented by one spring chain with at least 2 joints.', 'Add the named left ear bones to a VRMC_springBone spring and list them in profile.rig.earBonesL.');
    } else roleSpringIndexes.add(leftIndex);
    if (right.length < 2 || right.some((node) => node < 0) || rightIndex < 0) {
      add(diagnostics, 'A1.4-cat-ear-spring-bones', 'error', 'Right cat ear must be represented by one spring chain with at least 2 joints.', 'Add the named right ear bones to a VRMC_springBone spring and list them in profile.rig.earBonesR.');
    } else roleSpringIndexes.add(rightIndex);
    if (tail.length < 8 || tail.some((node) => node < 0) || tailIndex < 0) {
      add(diagnostics, 'A1.4-tail-spring-bones', 'error', 'The profile tail rig must resolve to one spring chain with at least 8 joints.', 'Use 8 or more ordered tail bones in profile.rig.tailBones and bind them to one spring.');
    } else roleSpringIndexes.add(tailIndex);
  } else {
    const earIndexes = identifySpringIndexes('ear');
    const tailIndexes = identifySpringIndexes('tail');
    if (earIndexes.length > 0 && earIndexes.some((index) => (jointArrays[index] ?? []).length < 2)) {
      add(diagnostics, 'A1.4-cat-ear-spring-bones', 'error', 'A detected cat ear spring chain has fewer than 2 joints.', 'Add at least 2 ordered joints to each ear spring chain.');
    } else if (earIndexes.length === 0) {
      add(diagnostics, 'A1.4-cat-ear-spring-bones', 'warning', 'Cat ear spring chains could not be identified without a profile.', 'Pass --profile nemu.profile.json so the ear bone names can be checked.');
    } else if (earIndexes.length < 2) {
      add(diagnostics, 'A1.4-cat-ear-spring-bones', 'warning', 'Only one cat ear spring chain was identified; both sides cannot be verified without a profile.', 'Pass --profile nemu.profile.json and list both ear chains.');
    }
    if (tailIndexes.length > 0 && tailIndexes.every((index) => (jointArrays[index] ?? []).length < 8)) {
      add(diagnostics, 'A1.4-tail-spring-bones', 'error', 'Detected tail spring chains have fewer than 8 joints.', 'Add an ordered tail chain with at least 8 joints.');
    } else if (tailIndexes.length === 0) {
      add(diagnostics, 'A1.4-tail-spring-bones', 'warning', 'Tail spring chain could not be identified without a profile.', 'Pass --profile nemu.profile.json so tail bones can be checked.');
    }
    [...earIndexes, ...tailIndexes].forEach((index) => roleSpringIndexes.add(index));
  }

  for (const [category, minimum] of [['hair', 6], ['skirt', 8]] as const) {
    const indexes = identifySpringIndexes(category);
    indexes.forEach((index) => roleSpringIndexes.add(index));
    const rule = category === 'hair' ? 'A1.4-hair-spring-bones' : 'A1.4-skirt-spring-bones';
    if (indexes.length === 0) {
      add(diagnostics, rule, 'warning', `${category} spring chains could not be identified from spring and bone names.`, `Name or configure at least ${minimum} ${category} spring chains so the validator can check them.`);
    } else if (indexes.length < minimum) {
      add(diagnostics, rule, 'warning', `${category} has ${indexes.length} identifiable spring chains; ${minimum} are required for Nemu.`, `Add at least ${minimum} ${category} spring chains.`);
    }
  }

  const colliderNodeIndices = new Map<number, Set<string>>();
  colliders.forEach((colliderValue) => {
    const collider = object(colliderValue);
    const nodeIndex = number(collider?.node);
    const shape = object(collider?.shape);
    if (nodeIndex === undefined || !shape) return;
    const set = colliderNodeIndices.get(nodeIndex) ?? new Set<string>();
    if (object(shape.sphere)) set.add('sphere');
    if (object(shape.capsule)) set.add('capsule');
    colliderNodeIndices.set(nodeIndex, set);
  });
  const requiredColliders = new Map<number, string>();
  const headNode = humanBones.get('head');
  if (headNode !== undefined) requiredColliders.set(headNode, 'sphere');
  COLLIDER_CAPSULE_BONES.forEach((bone) => {
    const node = humanBones.get(bone);
    if (node !== undefined) requiredColliders.set(node, 'capsule');
  });
  const missing = [...requiredColliders].filter(([node, shape]) => !colliderNodeIndices.get(node)?.has(shape));
  if (headNode === undefined || missing.length > 0) {
    const display = missing.map(([node, shape]) => `${names[node] ?? `node ${node}`} (${shape})`).join(', ');
    add(diagnostics, 'A1.4-required-colliders', 'error', `Required spring colliders are missing${display ? `: ${display}` : ' (head bone mapping unavailable)'}.`, 'Attach a sphere to head and capsules to neck, upperChest, both upper/lower arms, hips, and both upper legs.');
  }

  if (springs.length > 0) {
    const colliderIndicesByGroup = colliderGroups.map((groupValue) => array(object(groupValue)?.colliders).filter((index): index is number => typeof index === 'number'));
    const invalidColliderGroupIndexes = colliderIndicesByGroup.some((indices) => indices.some((index) => index < 0 || index >= colliders.length));
    if (invalidColliderGroupIndexes) {
      add(diagnostics, 'A1.4-spring-colliders', 'error', 'A collider group references a collider index that does not exist.', 'Correct every collider index to reference VRMC_springBone.colliders.');
    }
    const completeGroups = new Set<number>();
    colliderIndicesByGroup.forEach((indices, groupIndex) => {
      const nodesWithShapes = new Set(indices.map((index) => {
        const collider = object(colliders[index]);
        return `${number(collider?.node)}:${object(collider?.shape)?.sphere ? 'sphere' : object(collider?.shape)?.capsule ? 'capsule' : ''}`;
      }));
      if ([...requiredColliders].every(([node, shape]) => nodesWithShapes.has(`${node}:${shape}`))) completeGroups.add(groupIndex);
    });
    const invalidSpringGroupReference = [...roleSpringIndexes].find((springIndex) => {
      const springValue = springs[springIndex];
      const refs = array(object(springValue)?.colliderGroups).filter((index): index is number => typeof index === 'number');
      return refs.some((index) => index < 0 || index >= colliderGroups.length);
    });
    if (invalidSpringGroupReference !== undefined) {
      add(diagnostics, 'A1.4-spring-colliders', 'error', `Spring chain ${string(object(springs[invalidSpringGroupReference])?.name) ?? invalidSpringGroupReference} references a collider group that does not exist.`, 'Correct every spring colliderGroups reference to an existing VRMC_springBone.colliderGroups entry.');
    }
    const missingGroupSpring = [...roleSpringIndexes].find((springIndex) => {
      const springValue = springs[springIndex];
      const refs = array(object(springValue)?.colliderGroups).filter((index): index is number => typeof index === 'number');
      return refs.length === 0
        || !refs.some((index) => completeGroups.has(index));
    });
    if (missingGroupSpring !== undefined) {
      add(diagnostics, 'A1.4-spring-colliders', 'error', `Spring chain ${string(object(springs[missingGroupSpring])?.name) ?? missingGroupSpring} does not reference a collider group containing every required body collider.`, 'Create a collider group with all required colliders and reference it from each hair, ear, tail, and skirt spring.');
    }
  }
}

function findManifestPath(start: string): string | undefined {
  let directory = dirname(start);
  for (;;) {
    const candidate = resolve(directory, 'manifest.json');
    if (existsSync(candidate)) return candidate;
    const parent = dirname(directory);
    if (parent === directory) return undefined;
    directory = parent;
  }
}

async function loadContractSchemas(): Promise<typeof import('@nemu/contracts')> {
  const modulePath = resolve(import.meta.dirname, '../../../packages/contracts/dist/index.js');
  return await import(pathToFileURL(modulePath).href) as typeof import('@nemu/contracts');
}

async function validateProfile(path: string, modelPath: string, json: JsonObject, vrm: JsonObject, diagnostics: Diagnostic[]): Promise<NemuProfile | undefined> {
  let raw: unknown;
  try {
    raw = JSON.parse(await readFile(path, 'utf8')) as unknown;
  } catch (error) {
    add(diagnostics, 'A1.6-profile-json', 'error', `Could not read profile ${path}: ${String(error)}.`, 'Provide a valid nemu.profile.json file.');
    return undefined;
  }
  const { nemuProfileSchema } = await loadContractSchemas();
  const parsed = nemuProfileSchema.safeParse(raw);
  if (!parsed.success) {
    add(diagnostics, 'A1.6-profile-schema', 'error', `Profile does not match CONTRACTS.md 1.6: ${parsed.error.issues.map((issue) => `${issue.path.join('.') || '<root>'}: ${issue.message}`).join('; ')}.`, 'Regenerate the profile from build.yaml and the measured model data.');
    return undefined;
  }
  const names = new Set(nodeNames(json));
  humanBoneNodes(vrm).forEach((_, bone) => names.add(bone));
  const references = [
    ...parsed.data.hitRegions.map((region) => ['hitRegions', region.bone] as const),
    ['framing.fullBody', parsed.data.framing.fullBody.targetBone] as const,
    ['framing.upperBody', parsed.data.framing.upperBody.targetBone] as const,
    ...parsed.data.rig.earBonesL.map((bone) => ['rig.earBonesL', bone] as const),
    ...parsed.data.rig.earBonesR.map((bone) => ['rig.earBonesR', bone] as const),
    ...parsed.data.rig.tailBones.map((bone) => ['rig.tailBones', bone] as const),
  ];
  const missing = references.filter(([, bone]) => !names.has(bone));
  if (missing.length > 0) {
    add(diagnostics, 'A1.6-profile-bones', 'error', `Profile references missing model bones: ${missing.map(([field, bone]) => `${field}=${bone}`).join(', ')}.`, 'Use bone and node names present in the VRM.');
  }
  const profileModel = resolve(dirname(path), parsed.data.model);
  if (profileModel.toLowerCase() !== resolve(modelPath).toLowerCase()) {
    add(diagnostics, 'A1.6-profile-model', 'error', `Profile model path ${parsed.data.model} does not resolve to ${basename(modelPath)}.`, 'Set profile.model to the VRM path relative to the profile file.');
  }
  return parsed.data;
}

async function validateManifest(
  modelPath: string,
  explicitManifest: string | undefined,
  diagnostics: Diagnostic[],
): Promise<void> {
  const manifestPath = explicitManifest ? resolve(explicitManifest) : findManifestPath(modelPath);
  if (!manifestPath) {
    add(diagnostics, 'A1.5-manifest-missing', 'warning', 'No manifest.json was found in the model directory or its parents.', 'Register the model with source, SPDX license, and redistributable status in content/nemu/manifest.json.');
    return;
  }
  let raw: unknown;
  try {
    raw = JSON.parse(await readFile(manifestPath, 'utf8')) as unknown;
  } catch (error) {
    add(diagnostics, 'A1.5-manifest-json', 'error', `Could not read manifest ${manifestPath}: ${String(error)}.`, 'Repair the JSON manifest.');
    return;
  }
  const entriesValue = Array.isArray(raw) ? raw : array(object(raw)?.assets);
  if (!Array.isArray(raw) && !Array.isArray(object(raw)?.assets)) {
    add(diagnostics, 'A1.5-manifest-schema', 'error', 'Manifest must be an array of asset entries or an object with an assets array.', 'Use the content-template manifest shape and keep each entry aligned with AssetManifestEntry.');
    return;
  }
  const { assetManifestEntrySchema } = await loadContractSchemas();
  const validEntries: AssetManifestEntry[] = [];
  entriesValue.forEach((entry, index) => {
    const parsed = assetManifestEntrySchema.safeParse(entry);
    if (!parsed.success || (parsed.success && (!parsed.data.path.trim() || !parsed.data.source.trim() || !parsed.data.license.trim()))) {
      add(diagnostics, 'A1.5-manifest-entry', 'error', `Manifest entry ${index} lacks a valid source, license, kind, path, or redistributable value.`, 'Complete every required AssetManifestEntry field using an SPDX identifier or custom:<description> license.');
    } else {
      validEntries.push(parsed.data);
    }
  });
  const modelRelative = relative(dirname(manifestPath), modelPath).replaceAll('\\', '/');
  const hasModel = validEntries.some((entry) => entry.path.replaceAll('\\', '/') === modelRelative || basename(entry.path) === basename(modelPath));
  if (!hasModel) {
    add(diagnostics, 'A1.5-manifest-model', 'warning', `Manifest ${manifestPath} has no entry for ${basename(modelPath)}.`, 'Add a model manifest entry with source, license, and redistributable fields.');
  }
}

export async function lintVrm(options: LintOptions): Promise<LintResult> {
  const diagnostics: Diagnostic[] = [];
  const file = resolve(options.file);
  let json: JsonObject;
  let document: Document;
  let binaryChunks: Uint8Array[];
  try {
    ({ json, document, binaryChunks } = await readVrm(file));
  } catch (error) {
    add(diagnostics, 'A1.1-glb-parse', 'error', `Could not parse VRM/GLB: ${String(error)}.`, 'Export a complete GLB 2.0 VRM file and verify its binary chunks.');
    return finalize(file, diagnostics);
  }

  const asset = object(json.asset) ?? {};
  const extensions = getExtensions(json);
  const vrm = object(extensions.VRMC_vrm) ?? {};
  if (asset.version !== '2.0' || vrm.specVersion !== '1.0') {
    add(diagnostics, 'A1.1-vrm-1.0', 'error', `Expected glTF 2.0 with VRMC_vrm specVersion 1.0; found glTF ${String(asset.version)} and VRM ${String(vrm.specVersion)}.`, 'Export the model as VRM 1.0.');
  }

  const bounds = geometryBounds(json);
  if (!bounds) {
    add(diagnostics, 'A1.1-model-bounds', 'error', 'Could not measure model bounds from POSITION accessors.', 'Ensure all position accessors include valid min/max values and that mesh nodes reference them.');
  } else if (Math.abs(bounds.minY) > FLOOR_FLOAT_EPSILON) {
    add(diagnostics, 'A1.1-floor-y-zero', 'error', `Lowest measured model point is y=${bounds.minY.toFixed(6)} m; expected y=0.`, 'Move the model so the soles align with y=0 before export.');
  }
  const hipsNodeIndex = humanBoneNodes(vrm).get('hips');
  if (hipsNodeIndex === undefined) {
    add(diagnostics, 'A1.1-forward-positive-z', 'error', 'Cannot determine the model forward direction because the VRM has no humanoid hips mapping.', 'Provide a valid humanoid mapping and orient the avatar toward +Z.');
  } else {
    const nodes = array(json.nodes);
    // Use the avatar root frame; VRoid commonly tilts the hips bone slightly in its rest pose.
    const rootIndex = rootAncestorIndex(nodes, hipsNodeIndex);
    const rootWorld = nodeWorldMatrices(nodes)[rootIndex];
    if (!rootWorld) {
      add(diagnostics, 'A1.1-forward-positive-z', 'error', 'Humanoid root node index is outside the glTF node array.', 'Correct the VRM humanoid node mapping.');
    } else {
      const forwardZAxis = rootWorld[10] ?? 0;
      const forwardLength = Math.hypot(rootWorld[8] ?? 0, rootWorld[9] ?? 0, forwardZAxis);
      const forwardZ = forwardLength === 0 ? 0 : forwardZAxis / forwardLength;
      if (forwardZ < 0.999) {
        add(diagnostics, 'A1.1-forward-positive-z', 'error', `Humanoid hips forward axis maps to world Z=${forwardZ.toFixed(4)}; expected +Z.`, 'Rotate the avatar root so its humanoid forward axis faces +Z.');
      }
    }
  }

  const triangles = triangleCount(document);
  if (triangles > MAX_TRIANGLES) add(diagnostics, 'A1.1-triangle-budget', 'error', `${triangles} triangles exceed the 100,000 limit.`, 'Reduce mesh detail while preserving the model design.');
  else if (triangles > TARGET_TRIANGLES) add(diagnostics, 'A1.1-triangle-target', 'warning', `${triangles} triangles exceed the 70,000 target.`, 'Consider reducing mesh detail if it does not affect silhouette or expression.');

  const images = array(json.images);
  const imageSizes = new Map<number, { width: number; height: number }>();
  for (let index = 0; index < images.length; index += 1) {
    const bytes = await imageBytes(images[index], json, binaryChunks, dirname(file));
    const size = bytes ? dimensions(bytes) : undefined;
    if (!size || size.width < 1 || size.height < 1) {
      add(diagnostics, 'A1.1-texture-dimensions', 'error', `Could not read dimensions for image ${index}.`, 'Use an embedded PNG, JPEG, WebP VP8X, or KTX2 image with a valid header.');
      continue;
    }
    imageSizes.set(index, size);
    if (size.width > MAX_TEXTURE_EDGE || size.height > MAX_TEXTURE_EDGE) {
      add(diagnostics, 'A1.1-texture-edge', 'error', `Image ${index} is ${size.width}×${size.height}; each edge must be at most 2048 pixels.`, 'Resize the texture to 2048×2048 or smaller.');
    }
  }
  const textureBytes = [...imageSizes.values()].reduce((total, size) => total + size.width * size.height * 4 * MIP_FACTOR, 0);
  if (textureBytes > MAX_TEXTURE_BYTES) {
    add(diagnostics, 'A1.1-texture-vram', 'error', `RGBA8 plus mipmaps is ${(textureBytes / 1024 / 1024).toFixed(1)} MiB; the limit is 256 MiB.`, 'Reduce texture dimensions or remove unused image resources.');
  }
  if (array(json.textures).length > MAX_TEXTURE_COUNT) {
    add(diagnostics, 'A1.1-texture-count', 'warning', `${array(json.textures).length} textures exceed the 16-texture target.`, 'Combine or remove textures where the material appearance is preserved.');
  }
  if (array(json.materials).length > MAX_MATERIALS) {
    add(diagnostics, 'A1.1-material-count', 'warning', `${array(json.materials).length} materials exceed the 12-material target.`, 'Combine compatible materials where the appearance is preserved.');
  }
  const fileBytes = (await stat(file)).size;
  if (fileBytes > MAX_FILE_BYTES) add(diagnostics, 'A1.1-file-size', 'warning', `File size is ${(fileBytes / 1024 / 1024).toFixed(1)} MiB; the target is 60 MiB.`, 'Reduce unused resources or texture dimensions.');
  const boneCount = new Set(array(json.skins).flatMap((skin) => array(object(skin)?.joints).filter((joint): joint is number => typeof joint === 'number'))).size;
  if (boneCount > MAX_BONES) add(diagnostics, 'A1.1-bone-count', 'warning', `${boneCount} skin joints exceed the 250-bone target.`, 'Remove unused skin joints and preserve the required humanoid rig.');
  const morphCount = faceMorphCount(json, vrm);
  if (morphCount > MAX_FACE_MORPHS) add(diagnostics, 'A1.1-face-morph-count', 'warning', `${morphCount} morph targets on a mesh exceed the 120-key target.`, 'Remove unused face morph targets while preserving the required expressions.');

  array(json.materials).forEach((materialValue, index) => {
    const material = object(materialValue) ?? {};
    const materialExtensions = getExtensions(material);
    if (!object(materialExtensions[MTOON]) && !object(materialExtensions[UNLIT])) {
      add(diagnostics, 'A1.1-mtoon-material', 'error', `Material ${string(material.name) ?? index} is not MToon 1.0 or explicitly unlit.`, 'Assign VRMC_materials_mtoon or KHR_materials_unlit to every material.');
    }
  });

  const bones = humanBoneNodes(vrm);
  const missingBones = REQUIRED_HUMAN_BONES.filter((bone) => !bones.has(bone));
  if (missingBones.length > 0) add(diagnostics, 'A1.2-human-bones', 'error', `Required humanoid bones are missing: ${missingBones.join(', ')}.`, 'Add every required VRM 1.0 humanoid bone, including the specified torso, shoulder, toe, and finger bones.');

  const expressions = object(vrm.expressions) ?? {};
  const preset = findExpressionObject(expressions, 'preset');
  const absentPreset = REQUIRED_PRESET_EXPRESSIONS.filter((name) => !object(preset[name]));
  if (absentPreset.length > 0) add(diagnostics, 'A1.3-preset-expressions', 'error', `Required preset expressions are missing: ${absentPreset.join(', ')}.`, 'Add every preset expression listed in CONTRACTS.md 1.3.');
  const mouthNone = ['happy', 'angry', 'sad', 'relaxed', 'surprised'];
  mouthNone.filter((name) => object(preset[name])?.overrideMouth !== 'none').forEach((name) => {
    add(diagnostics, `A1.3-${name}-override`, 'error', `${name} must set overrideMouth to "none".`, 'Set overrideMouth to "none" so lip sync remains visible during speech.');
  });
  ['happy', 'relaxed'].filter((name) => object(preset[name])?.overrideBlink !== 'blend').forEach((name) => {
    add(diagnostics, `A1.3-${name}-override`, 'error', `${name} must set overrideBlink to "blend".`, 'Set overrideBlink to "blend".');
  });
  if (object(preset.surprised)?.overrideBlink !== 'block') add(diagnostics, 'A1.3-surprised-override', 'error', 'surprised must set overrideBlink to "block".', 'Set overrideBlink to "block".');
  ['aa', 'ih', 'ou', 'ee', 'oh'].filter((name) => object(preset[name])?.isBinary !== false).forEach((name) => {
    add(diagnostics, 'A1.3-viseme-nonbinary', 'error', `${name} must set isBinary to false.`, `Set ${name}.isBinary to false for interpolated lip sync.`);
  });
  const custom = findExpressionObject(expressions, 'custom');
  const missingCustom = CUSTOM_EXPRESSIONS.filter((name) => !object(custom[name]));
  if (missingCustom.length > 0) add(diagnostics, 'A1.3-custom-expressions', 'error', `Required Nemu expressions are missing: ${missingCustom.join(', ')}.`, 'Add the eight custom expressions listed in CONTRACTS.md 1.3.');
  if (object(vrm.lookAt)?.type !== 'bone') add(diagnostics, 'A1.3-look-at-bone', 'warning', 'VRM lookAt is not configured as a bone-type lookAt.', 'Use bone-type lookAt with a suitable angle mapping.');

  if (typeof options.profile === 'string') {
    const profile = await validateProfile(resolve(options.profile), file, json, vrm, diagnostics);
    if (profile) {
      if (bounds && Math.abs(profile.heightM - (bounds.maxY - bounds.minY)) > 0.02) {
        add(diagnostics, 'A1.1-profile-height', 'error', `Profile height ${profile.heightM.toFixed(3)} m differs from measured model height ${(bounds.maxY - bounds.minY).toFixed(3)} m by more than 0.02 m.`, 'Regenerate heightM from the model measurement.');
      }
      validateSprings(diagnostics, json, vrm, profile);
    }
  } else {
    validateSprings(diagnostics, json, vrm, undefined);
  }

  const metadata = object(vrm.meta) ?? {};
  if (metadata.name !== 'Nemu' || !string(metadata.licenseUrl)?.trim() || array(metadata.authors).length === 0) {
    add(diagnostics, 'A1.5-vrm-metadata', 'error', 'VRM meta must use name "Nemu" and include authors and licenseUrl.', 'Set meta.name to Nemu, add at least one author, and point licenseUrl to the content license file.');
  }
  await validateManifest(file, options.manifest, diagnostics);
  const metrics: LintMetrics = {
    nodeCount: array(json.nodes).length,
    meshCount: array(json.meshes).length,
    triangleCount: triangles,
    textureCount: array(json.textures).length,
    imageCount: images.length,
    materialCount: array(json.materials).length,
    estimatedTextureBytes: Math.round(textureBytes),
    skinJointCount: boneCount,
    maximumMeshMorphTargetCount: morphCount,
    ...(bounds ? { accessorBoundsHeightM: bounds.maxY - bounds.minY } : {}),
    fileBytes,
  };
  return finalize(file, diagnostics, metrics);
}

function finalize(file: string, diagnostics: Diagnostic[], metrics?: LintMetrics): LintResult {
  return {
    file,
    errors: diagnostics.filter((diagnostic) => diagnostic.severity === 'error').length,
    warnings: diagnostics.filter((diagnostic) => diagnostic.severity === 'warning').length,
    diagnostics,
    ...(metrics ? { metrics } : {}),
  };
}
