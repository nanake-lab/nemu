// CLI-only loader: the production GLTF/VRM plugins, without a browser or renderer.
import { readFile } from 'node:fs/promises';
import { Texture } from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { VRMLoaderPlugin } from '@pixiv/three-vrm';

export async function loadPhysicsVrm(path) {
  const bytes = await readFile(path);
  const loader = new GLTFLoader();
  // Texture pixels affect rendering only. Keep GLB nodes/skins/transforms and all
  // VRM spring data intact; don't decode images or create a WebGL context.
  loader.register(() => ({ name: 'P205_CLI_TEXTURE_PLACEHOLDER',
    loadTexture(index) { const texture = new Texture(); texture.name = `cli-unused-texture-${index}`; return Promise.resolve(texture); },
  }));
  loader.register(parser => new VRMLoaderPlugin(parser));
  const gltf = await loader.parseAsync(bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength), '');
  const vrm = gltf.userData.vrm;
  if (vrm?.meta.metaVersion !== '1' || !vrm.springBoneManager) throw new Error('Expected VRM 1.0 with spring bones');
  return { vrm, bytes };
}
