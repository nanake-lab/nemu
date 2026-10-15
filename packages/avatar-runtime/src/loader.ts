import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { VRMLoaderPlugin, VRM } from '@pixiv/three-vrm';
import { nemuProfileSchema, type NemuProfile } from '@nemu/contracts';

export interface AvatarLoadResult {
  vrm: VRM;
  profile: NemuProfile;
}

export class StageError extends Error {
  readonly code: string;

  constructor(code: string, message: string) {
    super(message);
    this.name = 'StageError';
    this.code = code;
  }
}

/**
 * Loads and validates a NemuProfile JSON object or URL.
 */
export async function loadProfile(profileSource: string | object): Promise<NemuProfile> {
  let rawData: unknown;
  if (typeof profileSource === 'string') {
    // If it looks like a JSON string, parse it; otherwise fetch from URL
    const trimmed = profileSource.trim();
    if (trimmed.startsWith('{')) {
      try {
        rawData = JSON.parse(trimmed);
      } catch (err) {
        throw new StageError(
          'INVALID_PROFILE_JSON',
          `Failed to parse profile JSON: ${(err as Error).message}`,
        );
      }
    } else {
      try {
        const response = await fetch(profileSource);
        if (!response.ok) {
          throw new StageError(
            'PROFILE_FETCH_FAILED',
            `HTTP ${response.status} fetching profile from ${profileSource}`,
          );
        }
        rawData = await response.json();
      } catch (err) {
        if (err instanceof StageError) throw err;
        throw new StageError(
          'PROFILE_FETCH_FAILED',
          `Failed to fetch profile: ${(err as Error).message}`,
        );
      }
    }
  } else {
    rawData = profileSource;
  }

  const result = nemuProfileSchema.safeParse(rawData);
  if (!result.success) {
    const errorDetails = result.error.issues
      .map((e) => `${e.path.join('.')}: ${e.message}`)
      .join(', ');
    throw new StageError(
      'INVALID_PROFILE_SCHEMA',
      `Profile schema validation failed: ${errorDetails}`,
    );
  }

  return result.data as NemuProfile;
}

/**
 * Loads a VRM 1.0 avatar using GLTFLoader and VRMLoaderPlugin.
 * Directly rejects with StageError('UNSUPPORTED_VRM_VERSION') if the model is VRM 0.x.
 */
export async function loadVRM(vrmSource: string | ArrayBuffer): Promise<VRM> {
  const loader = new GLTFLoader();
  loader.register((parser) => new VRMLoaderPlugin(parser));

  return new Promise<VRM>((resolve, reject) => {
    const onLoad = (gltf: unknown) => {
      const vrm = (gltf as { userData: { vrm?: VRM } }).userData.vrm;
      if (!vrm) {
        reject(new StageError('NOT_A_VRM_MODEL', 'The loaded asset does not contain VRM data.'));
        return;
      }

      // Check VRM version. VRM 0.x has metaVersion === '0', VRM 1.0 has '1'.
      const metaVersion = vrm.meta?.metaVersion;
      if (metaVersion === '0' || metaVersion !== '1') {
        reject(
          new StageError(
            'UNSUPPORTED_VRM_VERSION',
            `VRM version ${metaVersion ?? 'unknown'} is not supported; only VRM 1.0 is supported.`,
          ),
        );
        return;
      }

      resolve(vrm);
    };

    const onError = (error: unknown) => {
      reject(
        new StageError(
          'VRM_LOAD_FAILED',
          `Failed to load VRM: ${(error as Error).message || String(error)}`,
        ),
      );
    };

    if (typeof vrmSource === 'string') {
      loader.load(vrmSource, onLoad, undefined, onError);
    } else {
      loader.parse(vrmSource, '', onLoad, onError);
    }
  });
}

/**
 * Loads both the VRM 1.0 avatar and its associated profile.
 */
export async function loadAvatar(
  vrmSource: string | ArrayBuffer,
  profileSource: string | object,
): Promise<AvatarLoadResult> {
  const [profile, vrm] = await Promise.all([loadProfile(profileSource), loadVRM(vrmSource)]);
  return { vrm, profile };
}
