import * as THREE from 'three';

export interface LightConfig {
  color: THREE.ColorRepresentation;
  intensity: number;
  position?: [number, number, number];
}

export interface ThreePointLightingConfig {
  keyLight: LightConfig;
  rimLight: LightConfig;
  ambientLight: LightConfig;
}

export const DEFAULT_LIGHTING_CONFIG: ThreePointLightingConfig = {
  keyLight: {
    color: 0xffffff,
    intensity: 1.1,
    position: [1.2, 2.0, 2.0],
  },
  rimLight: {
    color: 0xe8f0ff,
    intensity: 0.7,
    position: [-1.5, 1.8, -2.0],
  },
  ambientLight: {
    color: 0xffffff,
    intensity: 0.45,
  },
};

export class ThreePointLighting {
  readonly group: THREE.Group;
  readonly keyLight: THREE.DirectionalLight;
  readonly rimLight: THREE.DirectionalLight;
  readonly ambientLight: THREE.AmbientLight;

  constructor(config: Partial<ThreePointLightingConfig> = {}) {
    this.group = new THREE.Group();
    this.group.name = 'ThreePointLighting';

    const merged = {
      keyLight: { ...DEFAULT_LIGHTING_CONFIG.keyLight, ...config.keyLight },
      rimLight: { ...DEFAULT_LIGHTING_CONFIG.rimLight, ...config.rimLight },
      ambientLight: { ...DEFAULT_LIGHTING_CONFIG.ambientLight, ...config.ambientLight },
    };

    // 1. Key Light (Main front-angled light)
    this.keyLight = new THREE.DirectionalLight(merged.keyLight.color, merged.keyLight.intensity);
    if (merged.keyLight.position) {
      this.keyLight.position.set(...merged.keyLight.position);
    }
    this.keyLight.name = 'KeyLight';
    this.group.add(this.keyLight);

    // 2. Rim Light (Back-angled silhouette kicker)
    this.rimLight = new THREE.DirectionalLight(merged.rimLight.color, merged.rimLight.intensity);
    if (merged.rimLight.position) {
      this.rimLight.position.set(...merged.rimLight.position);
    }
    this.rimLight.name = 'RimLight';
    this.group.add(this.rimLight);

    // 3. Ambient Light (Soft neutral fill)
    this.ambientLight = new THREE.AmbientLight(
      merged.ambientLight.color,
      merged.ambientLight.intensity,
    );
    this.ambientLight.name = 'AmbientLight';
    this.group.add(this.ambientLight);
  }

  setKeyLight(
    intensity?: number,
    color?: THREE.ColorRepresentation,
    position?: [number, number, number],
  ): void {
    if (intensity !== undefined) this.keyLight.intensity = intensity;
    if (color !== undefined) this.keyLight.color.set(color);
    if (position !== undefined) this.keyLight.position.set(...position);
  }

  setRimLight(
    intensity?: number,
    color?: THREE.ColorRepresentation,
    position?: [number, number, number],
  ): void {
    if (intensity !== undefined) this.rimLight.intensity = intensity;
    if (color !== undefined) this.rimLight.color.set(color);
    if (position !== undefined) this.rimLight.position.set(...position);
  }

  setAmbientLight(intensity?: number, color?: THREE.ColorRepresentation): void {
    if (intensity !== undefined) this.ambientLight.intensity = intensity;
    if (color !== undefined) this.ambientLight.color.set(color);
  }

  getConfig(): ThreePointLightingConfig {
    return {
      keyLight: {
        color: this.keyLight.color.getHex(),
        intensity: this.keyLight.intensity,
        position: [this.keyLight.position.x, this.keyLight.position.y, this.keyLight.position.z],
      },
      rimLight: {
        color: this.rimLight.color.getHex(),
        intensity: this.rimLight.intensity,
        position: [this.rimLight.position.x, this.rimLight.position.y, this.rimLight.position.z],
      },
      ambientLight: {
        color: this.ambientLight.color.getHex(),
        intensity: this.ambientLight.intensity,
      },
    };
  }

  dispose(): void {
    this.keyLight.dispose();
    this.rimLight.dispose();
    this.ambientLight.dispose();
  }
}
