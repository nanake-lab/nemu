import * as THREE from 'three';

export type AntialiasMode = 'defaultMSAA' | 'renderTarget4x';

export interface AvatarRendererOptions {
  canvas?: HTMLCanvasElement;
  width?: number;
  height?: number;
  antialiasMode?: AntialiasMode;
  devicePixelRatio?: number;
}

export class AvatarRenderer {
  readonly renderer: THREE.WebGLRenderer;
  private antialiasMode: AntialiasMode;
  private msaaRenderTarget: THREE.WebGLRenderTarget | null = null;
  private renderWidth = 0;
  private renderHeight = 0;

  constructor(options: AvatarRendererOptions = {}) {
    this.antialiasMode = options.antialiasMode ?? 'defaultMSAA';

    this.renderer = new THREE.WebGLRenderer({
      canvas: options.canvas,
      alpha: true,
      premultipliedAlpha: true,
      antialias: true,
      preserveDrawingBuffer: true,
      powerPreference: 'high-performance',
    });

    // Color management according to three-vrm recommendations
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    this.renderer.toneMapping = THREE.LinearToneMapping;

    if (options.devicePixelRatio !== undefined) {
      this.renderer.setPixelRatio(options.devicePixelRatio);
    } else if (typeof window !== 'undefined') {
      this.renderer.setPixelRatio(window.devicePixelRatio || 1);
    }

    if (options.width && options.height) {
      this.setSize(options.width, options.height);
    }
  }

  setAntialiasMode(mode: AntialiasMode): void {
    if (this.antialiasMode === mode) return;
    this.antialiasMode = mode;
    this.updateRenderTarget();
  }

  getAntialiasMode(): AntialiasMode {
    return this.antialiasMode;
  }

  setSize(width: number, height: number, updateStyle = true): void {
    this.renderWidth = width;
    this.renderHeight = height;
    this.renderer.setSize(width, height, updateStyle);
    this.updateRenderTarget();
  }

  private updateRenderTarget(): void {
    if (this.antialiasMode === 'renderTarget4x') {
      if (this.msaaRenderTarget) {
        this.msaaRenderTarget.dispose();
      }
      const pixelRatio = this.renderer.getPixelRatio();
      const actualWidth = Math.floor(this.renderWidth * pixelRatio);
      const actualHeight = Math.floor(this.renderHeight * pixelRatio);

      if (actualWidth > 0 && actualHeight > 0) {
        this.msaaRenderTarget = new THREE.WebGLRenderTarget(actualWidth, actualHeight, {
          format: THREE.RGBAFormat,
          type: THREE.UnsignedByteType,
          samples: 4,
          colorSpace: THREE.SRGBColorSpace,
        });
      }
    } else {
      if (this.msaaRenderTarget) {
        this.msaaRenderTarget.dispose();
        this.msaaRenderTarget = null;
      }
    }
  }

  render(scene: THREE.Scene, camera: THREE.Camera): void {
    if (this.antialiasMode === 'renderTarget4x' && this.msaaRenderTarget) {
      // Render to 4x MSAA render target, then blit to screen
      this.renderer.setRenderTarget(this.msaaRenderTarget);
      this.renderer.clear();
      this.renderer.render(scene, camera);

      this.renderer.setRenderTarget(null);
      // Blit texture to canvas via Three.js internal render or full-screen quad if needed,
      // or use framebuffer copy
      // Note: for WebGL2, blitFramebuffer copies between MSAA render target and default framebuffer
      const gl = this.renderer.getContext() as WebGL2RenderingContext;
      if (gl && this.msaaRenderTarget) {
        const renderTargetProps = this.renderer.properties.get(this.msaaRenderTarget) as {
          __webglFramebuffer?: WebGLFramebuffer;
        };
        const srcFb = renderTargetProps?.__webglFramebuffer;
        if (srcFb) {
          const w = this.msaaRenderTarget.width;
          const h = this.msaaRenderTarget.height;
          gl.bindFramebuffer(gl.READ_FRAMEBUFFER, srcFb);
          gl.bindFramebuffer(gl.DRAW_FRAMEBUFFER, null);
          gl.blitFramebuffer(0, 0, w, h, 0, 0, w, h, gl.COLOR_BUFFER_BIT, gl.NEAREST);
          gl.bindFramebuffer(gl.READ_FRAMEBUFFER, null);
        } else {
          // Fallback direct render
          this.renderer.render(scene, camera);
        }
      }
    } else {
      // Default framebuffer MSAA
      this.renderer.setRenderTarget(null);
      this.renderer.render(scene, camera);
    }
  }

  getCanvas(): HTMLCanvasElement {
    return this.renderer.domElement;
  }

  dispose(): void {
    if (this.msaaRenderTarget) {
      this.msaaRenderTarget.dispose();
      this.msaaRenderTarget = null;
    }
    this.renderer.dispose();
  }
}
