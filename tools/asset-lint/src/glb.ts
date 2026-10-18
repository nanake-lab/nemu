import { NodeIO } from '@gltf-transform/core';
import { readFile } from 'node:fs/promises';

export type JsonObject = Record<string, unknown>;

export interface ParsedGlb {
  json: JsonObject;
  binaryChunks: Uint8Array[];
  document: Awaited<ReturnType<NodeIO['readBinary']>>;
}

function isObject(value: unknown): value is JsonObject {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function makeGlb(json: JsonObject, binaryChunks: Uint8Array[]): Uint8Array {
  const encodedJson = Buffer.from(JSON.stringify(json), 'utf8');
  const paddedJsonLength = Math.ceil(encodedJson.byteLength / 4) * 4;
  const jsonChunk = Buffer.alloc(paddedJsonLength, 0x20);
  encodedJson.copy(jsonChunk);
  const chunks = [jsonChunk, ...binaryChunks];
  const chunkBytes = chunks.reduce((total, chunk) => total + 8 + chunk.byteLength, 0);
  const result = Buffer.alloc(12 + chunkBytes);
  result.writeUInt32LE(0x46546c67, 0);
  result.writeUInt32LE(2, 4);
  result.writeUInt32LE(result.byteLength, 8);
  let offset = 12;
  chunks.forEach((chunk, index) => {
    result.writeUInt32LE(chunk.byteLength, offset);
    result.writeUInt32LE(index === 0 ? 0x4e4f534a : 0x004e4942, offset + 4);
    Buffer.from(chunk).copy(result, offset + 8);
    offset += 8 + chunk.byteLength;
  });
  return result;
}

function stripExtensions(value: unknown): void {
  if (Array.isArray(value)) {
    value.forEach(stripExtensions);
    return;
  }
  if (!isObject(value)) return;
  delete value.extensions;
  delete value.extensionsUsed;
  delete value.extensionsRequired;
  Object.values(value).forEach(stripExtensions);
}

export function parseGlb(bytes: Uint8Array): { json: JsonObject; binaryChunks: Uint8Array[] } {
  const buffer = Buffer.from(bytes);
  if (buffer.byteLength < 20 || buffer.readUInt32LE(0) !== 0x46546c67) {
    throw new Error('Input is not a complete GLB file.');
  }
  if (buffer.readUInt32LE(4) !== 2) throw new Error('GLB container version must be 2.');
  if (buffer.readUInt32LE(8) !== buffer.byteLength) throw new Error('GLB declared length does not match the file length.');

  let offset = 12;
  let json: JsonObject | undefined;
  const binaryChunks: Uint8Array[] = [];
  while (offset < buffer.byteLength) {
    if (offset + 8 > buffer.byteLength) throw new Error('GLB has a truncated chunk header.');
    const chunkLength = buffer.readUInt32LE(offset);
    const chunkType = buffer.readUInt32LE(offset + 4);
    const chunkStart = offset + 8;
    const chunkEnd = chunkStart + chunkLength;
    if (chunkEnd > buffer.byteLength) throw new Error('GLB has a truncated chunk body.');
    const chunk = buffer.subarray(chunkStart, chunkEnd);
    if (chunkType === 0x4e4f534a) {
      if (json !== undefined) throw new Error('GLB contains more than one JSON chunk.');
      const decoded: unknown = JSON.parse(chunk.toString('utf8').replace(/ +$/u, ''));
      if (!isObject(decoded)) throw new Error('GLB JSON chunk must contain an object.');
      json = decoded;
    } else if (chunkType === 0x004e4942) {
      binaryChunks.push(chunk);
    }
    offset = chunkEnd;
  }
  if (json === undefined) throw new Error('GLB has no JSON chunk.');
  return { json, binaryChunks };
}

export async function readVrm(path: string): Promise<ParsedGlb> {
  const bytes = await readFile(path);
  const { json, binaryChunks } = parseGlb(bytes);
  const coreJson = structuredClone(json);
  stripExtensions(coreJson);
  const document = await new NodeIO().readBinary(makeGlb(coreJson, binaryChunks));
  return { json, binaryChunks, document };
}
