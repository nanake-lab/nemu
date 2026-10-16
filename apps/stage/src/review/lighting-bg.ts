import { ThreePointLighting } from '@nemu/avatar-runtime';

export type LightingPreset = 'threePoint' | 'studio' | 'dramatic' | 'dim' | 'flat';
export type BackgroundPreset = 'checker' | 'white' | 'black' | 'gradient' | 'wallpaper';

export class LightingAndBackgroundManager {
  private readonly lighting: ThreePointLighting;
  private readonly container: HTMLElement;

  private currentLightPreset: LightingPreset = 'threePoint';
  private currentBgPreset: BackgroundPreset = 'gradient';

  constructor(lighting: ThreePointLighting, container: HTMLElement) {
    this.lighting = lighting;
    this.container = container;
    this.applyBackground('gradient');
    this.applyLighting('threePoint');
  }

  applyLighting(preset: LightingPreset): void {
    this.currentLightPreset = preset;
    switch (preset) {
      case 'threePoint':
        this.lighting.setKeyLight(1.2, 0xffffff, [1.5, 2.0, 2.5]);
        this.lighting.setRimLight(0.8, 0xd0e0ff, [-2.0, 2.5, -2.0]);
        this.lighting.setAmbientLight(0.4, 0xffffff);
        break;
      case 'studio':
        this.lighting.setKeyLight(1.5, 0xffffff, [0.5, 2.5, 3.0]);
        this.lighting.setRimLight(0.4, 0xffffff, [-1.5, 1.5, -2.0]);
        this.lighting.setAmbientLight(0.6, 0xfafafa);
        break;
      case 'dramatic':
        this.lighting.setKeyLight(0.8, 0xffe0b2, [2.5, 1.5, 1.5]);
        this.lighting.setRimLight(1.6, 0x80d8ff, [-2.5, 2.5, -2.0]);
        this.lighting.setAmbientLight(0.15, 0x303050);
        break;
      case 'dim':
        this.lighting.setKeyLight(0.5, 0xffcc80, [1.0, 1.0, 2.0]);
        this.lighting.setRimLight(0.3, 0x9fa8da, [-1.5, 2.0, -1.5]);
        this.lighting.setAmbientLight(0.2, 0x212121);
        break;
      case 'flat':
        this.lighting.setKeyLight(0.6, 0xffffff, [0, 2.0, 3.0]);
        this.lighting.setRimLight(0.2, 0xffffff, [0, 2.0, -3.0]);
        this.lighting.setAmbientLight(0.85, 0xffffff);
        break;
    }
  }

  applyBackground(preset: BackgroundPreset): void {
    this.currentBgPreset = preset;
    this.container.classList.remove(
      'bg-checker',
      'bg-white',
      'bg-black',
      'bg-gradient',
      'bg-wallpaper',
    );
    this.container.classList.add(`bg-${preset}`);
  }

  getCurrentLighting(): LightingPreset {
    return this.currentLightPreset;
  }

  getCurrentBackground(): BackgroundPreset {
    return this.currentBgPreset;
  }
}
