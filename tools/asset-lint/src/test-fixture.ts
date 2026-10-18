import { mkdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';

type JsonRecord = Record<string, unknown>;

export interface FixtureOptions {
  triangleCount?: number;
  imageDimensions?: Array<{ width: number; height: number }>;
  mutate?: (json: JsonRecord) => void;
  mutateProfile?: (profile: JsonRecord) => void;
  manifest?: unknown;
  fileName?: string;
}

export interface FixturePaths {
  file: string;
  profile: string;
  manifest: string;
}

const HUMAN_BONES = [
  'hips', 'spine', 'head', 'leftUpperLeg', 'leftLowerLeg', 'leftFoot', 'rightUpperLeg',
  'rightLowerLeg', 'rightFoot', 'leftUpperArm', 'leftLowerArm', 'leftHand',
  'rightUpperArm', 'rightLowerArm', 'rightHand', 'chest', 'upperChest', 'neck',
  'leftShoulder', 'rightShoulder', 'leftToes', 'rightToes',
  ...['left', 'right'].flatMap((side) => ['Thumb', 'Index', 'Middle', 'Ring', 'Little'].flatMap((finger) => {
    const parts = finger === 'Thumb' ? ['Metacarpal', 'Proximal', 'Distal'] : ['Proximal', 'Intermediate', 'Distal'];
    return parts.map((part) => `${side}${finger}${part}`);
  })),
];
const EMOTIONS = ['neutral', 'happy', 'angry', 'sad', 'relaxed', 'surprised', 'smug', 'shy', 'sleepy', 'pout', 'excited', 'worried'];

function glb(json: JsonRecord, binary: Uint8Array): Buffer {
  const jsonBytes = Buffer.from(JSON.stringify(json), 'utf8');
  const jsonLength = Math.ceil(jsonBytes.length / 4) * 4;
  const paddedJson = Buffer.alloc(jsonLength, 0x20);
  jsonBytes.copy(paddedJson);
  const binaryBytes = Buffer.from(binary);
  const binaryLength = Math.ceil(binaryBytes.length / 4) * 4;
  const paddedBinary = Buffer.alloc(binaryLength);
  binaryBytes.copy(paddedBinary);
  const totalLength = 12 + 8 + paddedJson.length + 8 + paddedBinary.length;
  const result = Buffer.alloc(totalLength);
  result.writeUInt32LE(0x46546c67, 0);
  result.writeUInt32LE(2, 4);
  result.writeUInt32LE(totalLength, 8);
  result.writeUInt32LE(paddedJson.length, 12);
  result.writeUInt32LE(0x4e4f534a, 16);
  paddedJson.copy(result, 20);
  const binaryHeader = 20 + paddedJson.length;
  result.writeUInt32LE(paddedBinary.length, binaryHeader);
  result.writeUInt32LE(0x004e4942, binaryHeader + 4);
  paddedBinary.copy(result, binaryHeader + 8);
  return result;
}

function pngHeader(width: number, height: number): Buffer {
  const bytes = Buffer.alloc(24);
  Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]).copy(bytes, 0);
  bytes.writeUInt32BE(13, 8);
  bytes.write('IHDR', 12, 'ascii');
  bytes.writeUInt32BE(width, 16);
  bytes.writeUInt32BE(height, 20);
  return bytes;
}

function requiredColliders(humanBoneMap: Record<string, { node: number }>): { node: number; shape: JsonRecord }[] {
  const nodeFor = (bone: string): number => {
    const entry = humanBoneMap[bone];
    if (!entry) throw new Error(`Missing fixture bone mapping: ${bone}`);
    return entry.node;
  };
  const capsuleBones = ['neck', 'upperChest', 'leftUpperArm', 'rightUpperArm', 'leftLowerArm', 'rightLowerArm', 'hips', 'leftUpperLeg', 'rightUpperLeg'];
  const values: { node: number; shape: JsonRecord }[] = [
    { node: nodeFor('head'), shape: { sphere: { offset: [0, 0, 0], radius: 0.1 } } },
  ];
  capsuleBones.forEach((bone) => values.push({
    node: nodeFor(bone),
    shape: { capsule: { offset: [0, 0, 0], tail: [0, -0.1, 0], radius: 0.08 } },
  }));
  return values;
}

export async function createFixture(directory: string, options: FixtureOptions = {}): Promise<FixturePaths> {
  await mkdir(directory, { recursive: true });
  const fileName = options.fileName ?? 'fixture.vrm';
  const file = join(directory, fileName);
  const profilePath = join(directory, 'nemu.profile.json');
  const manifestPath = join(directory, 'manifest.json');

  const nodes: JsonRecord[] = [];
  const humanBoneMap: Record<string, { node: number }> = {};
  HUMAN_BONES.forEach((bone, index) => {
    humanBoneMap[bone] = { node: index };
    const y = bone === 'hips' ? 0.9 : bone === 'head' ? 1.6 : bone.endsWith('Foot') || bone.endsWith('Toes') ? 0 : 0.8;
    nodes.push({ name: bone, translation: [0, y, 0] });
  });
  const rigNames = {
    earBonesL: ['EarL1', 'EarL2'],
    earBonesR: ['EarR1', 'EarR2'],
    tailBones: Array.from({ length: 8 }, (_, index) => `Tail${index + 1}`),
  };
  [...rigNames.earBonesL, ...rigNames.earBonesR, ...rigNames.tailBones,
    ...Array.from({ length: 6 }, (_, index) => `Hair${index + 1}`),
    ...Array.from({ length: 8 }, (_, index) => `Skirt${index + 1}`),
  ].forEach((name) => nodes.push({ name }));
  const meshNode = nodes.length;
  nodes.push({ name: 'BodyMesh', mesh: 0 });

  const preset: JsonRecord = Object.fromEntries(['neutral', 'happy', 'angry', 'sad', 'relaxed', 'surprised', 'aa', 'ih', 'ou', 'ee', 'oh', 'blink', 'blinkLeft', 'blinkRight'].map((name) => [name, {}]));
  ['happy', 'angry', 'sad', 'relaxed', 'surprised'].forEach((name) => { preset[name] = { overrideMouth: 'none' }; });
  ['happy', 'relaxed'].forEach((name) => { preset[name] = { ...preset[name] as JsonRecord, overrideBlink: 'blend' }; });
  preset.surprised = { ...preset.surprised as JsonRecord, overrideBlink: 'block' };
  ['aa', 'ih', 'ou', 'ee', 'oh'].forEach((name) => { preset[name] = { isBinary: false }; });
  const custom = Object.fromEntries(['nemuSmug', 'nemuCatMouth', 'nemuSparkle', 'nemuBlush', 'nemuTeary', 'nemuSleepy', 'nemuPout', 'nemuAngryMark'].map((name) => [name, {}]));
  const colliders = requiredColliders(humanBoneMap);
  const springs = [
    { name: 'ear-left', joints: rigNames.earBonesL.map((name) => ({ node: nodes.findIndex((node) => node.name === name) })), colliderGroups: [0] },
    { name: 'ear-right', joints: rigNames.earBonesR.map((name) => ({ node: nodes.findIndex((node) => node.name === name) })), colliderGroups: [0] },
    { name: 'tail', joints: rigNames.tailBones.map((name) => ({ node: nodes.findIndex((node) => node.name === name) })), colliderGroups: [0] },
    ...Array.from({ length: 6 }, (_, index) => ({ name: `hair-${index}`, joints: [{ node: nodes.findIndex((node) => node.name === `Hair${index + 1}`) }, { node: nodes.findIndex((node) => node.name === `Hair${index + 1}`) }], colliderGroups: [0] })),
    ...Array.from({ length: 8 }, (_, index) => ({ name: `skirt-${index}`, joints: [{ node: nodes.findIndex((node) => node.name === `Skirt${index + 1}`) }, { node: nodes.findIndex((node) => node.name === `Skirt${index + 1}`) }], colliderGroups: [0] })),
  ];

  const positionBytes = Buffer.alloc(36);
  [0, 0, 0, 0, 1.7, 0, 0.1, 0.1, 0].forEach((value, index) => positionBytes.writeFloatLE(value, index * 4));
  const requestedTriangles = options.triangleCount ?? 1;
  const indexCount = requestedTriangles * 3;
  const indexBytes = Buffer.alloc(indexCount * 2);
  for (let index = 0; index < indexCount; index += 1) indexBytes.writeUInt16LE(index % 3, index * 2);
  const binaryParts: Uint8Array[] = [positionBytes, indexBytes];
  const bufferViews: JsonRecord[] = [
    { buffer: 0, byteOffset: 0, byteLength: positionBytes.length, target: 34962 },
    { buffer: 0, byteOffset: positionBytes.length, byteLength: indexBytes.length, target: 34963 },
  ];
  const images: JsonRecord[] = [];
  const textureDimensions = options.imageDimensions ?? [];
  textureDimensions.forEach((size) => {
    const bytes = pngHeader(size.width, size.height);
    const offset = binaryParts.reduce((total, part) => total + part.length, 0);
    bufferViews.push({ buffer: 0, byteOffset: offset, byteLength: bytes.length });
    images.push({ name: `texture-${images.length}`, bufferView: bufferViews.length - 1, mimeType: 'image/png' });
    binaryParts.push(bytes);
  });
  const binary = Buffer.concat(binaryParts);

  const json: JsonRecord = {
    asset: { version: '2.0', generator: 'Nemu asset-lint test fixture' },
    extensionsUsed: ['VRMC_vrm', 'VRMC_springBone', 'VRMC_materials_mtoon'],
    scene: 0,
    scenes: [{ nodes: [meshNode] }],
    nodes,
    meshes: [{ name: 'Body', primitives: [{ attributes: { POSITION: 0 }, indices: 1, material: 0 }] }],
    materials: [{ name: 'Toon', extensions: { VRMC_materials_mtoon: {} } }],
    accessors: [
      { bufferView: 0, componentType: 5126, count: 3, type: 'VEC3', min: [0, 0, 0], max: [0.1, 1.7, 0] },
      { bufferView: 1, byteOffset: 0, componentType: 5123, count: indexCount, type: 'SCALAR', min: [0], max: [2] },
    ],
    bufferViews,
    buffers: [{ byteLength: binary.length }],
    skins: [{ joints: Object.values(humanBoneMap).map((bone) => bone.node), skeleton: humanBoneMap.hips?.node ?? 0 }],
    images,
    textures: images.map((_, index) => ({ source: index })),
    extensions: {
      VRMC_vrm: {
        specVersion: '1.0',
        meta: { name: 'Nemu', authors: ['Nemu Team'], licenseUrl: '../LICENSE.md' },
        humanoid: { humanBones: humanBoneMap },
        expressions: { preset, custom },
        lookAt: { type: 'bone', rangeMapHorizontalInner: { inputMaxValue: 90, outputScale: 1 } },
      },
      VRMC_springBone: {
        colliders,
        colliderGroups: [{ name: 'body', colliders: colliders.map((_, index) => index) }],
        springs,
      },
    },
  };
  options.mutate?.(json);
  await writeFile(file, glb(json, binary));

  const profile: JsonRecord = {
    version: 2,
    model: fileName,
    heightM: 1.7,
    headTopOffsetM: 0.1,
    hitRegions: [{ id: 'head', bone: 'head', shape: 'sphere', radiusM: 0.1 }],
    gazeLimits: { yawDeg: 45, pitchUpDeg: 30, pitchDownDeg: 30 },
    framing: {
      fullBody: { fovDeg: 20, distanceM: 3, targetBone: 'head' },
      upperBody: { fovDeg: 20, distanceM: 1.5, targetBone: 'head' },
    },
    rig: { ...rigNames, earBackMaxDeg: 25, tailSwingDeg: [-20, 20] },
    emotionMap: Object.fromEntries(EMOTIONS.map((emotion) => [emotion, []])),
    visemeMap: { A: 'aa', I: 'ih', U: 'ou', E: 'ee', O: 'oh' },
    lipSyncProfiles: { zh: 'lipsync/zh.json', en: 'lipsync/en.json', ja: 'lipsync/ja.json' },
    idleSet: [{ motionId: 'idle_stand', weight: 1, cooldownS: 5 }],
  };
  options.mutateProfile?.(profile);
  await writeFile(profilePath, JSON.stringify(profile, null, 2), 'utf8');
  const manifest = options.manifest ?? [{ path: fileName, kind: 'model', source: 'Generated test fixture', license: 'CC0-1.0', redistributable: true }];
  await writeFile(manifestPath, JSON.stringify(manifest, null, 2), 'utf8');
  return { file, profile: profilePath, manifest: manifestPath };
}
