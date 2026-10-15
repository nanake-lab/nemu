import type * as THREE from 'three';

export interface HitTestOptions {
  alphaThreshold?: number; // default 0.1 (25 / 255)
  maxFrequencyHz?: number; // default 30 Hz
  onHitTestChange?: (overCharacter: boolean) => void;
}

export class HitTestManager {
  private readonly gl: WebGL2RenderingContext | null = null;
  private readonly pbo: WebGLBuffer | null = null;
  private readonly pixelBuffer = new Uint8Array(4);

  private currentSync: WebGLSync | null = null;
  private isReadPending = false;
  private lastCursorX = 0;
  private lastCursorY = 0;
  private lastCheckTime = 0;

  private isOverCharacter = false;
  private readonly alphaThreshold: number;
  private readonly minIntervalMs: number;
  private readonly onHitTestChange?: ((overCharacter: boolean) => void) | undefined;

  constructor(renderer: THREE.WebGLRenderer, options: HitTestOptions = {}) {
    this.alphaThreshold = options.alphaThreshold ?? 0.1;
    this.minIntervalMs = 1000 / (options.maxFrequencyHz ?? 30);
    this.onHitTestChange = options.onHitTestChange;

    const context = renderer.getContext();
    if (
      typeof WebGL2RenderingContext !== 'undefined' &&
      context instanceof WebGL2RenderingContext
    ) {
      this.gl = context;
      this.pbo = this.gl.createBuffer();
      this.gl.bindBuffer(this.gl.PIXEL_PACK_BUFFER, this.pbo);
      this.gl.bufferData(this.gl.PIXEL_PACK_BUFFER, 4, this.gl.STREAM_READ);
      this.gl.bindBuffer(this.gl.PIXEL_PACK_BUFFER, null);
    }
  }

  /**
   * Called on mousemove with canvas-relative coordinates.
   * Throttled to maxFrequencyHz (30Hz).
   */
  onMouseMove(canvasX: number, canvasY: number, canvasHeight: number): void {
    this.lastCursorX = Math.floor(canvasX);
    // WebGL origin is bottom-left, DOM origin is top-left
    this.lastCursorY = Math.floor(canvasHeight - canvasY);

    const now = performance.now();
    if (now - this.lastCheckTime >= this.minIntervalMs && !this.isReadPending) {
      this.requestPixelRead();
      this.lastCheckTime = now;
    }
  }

  /**
   * Dispatches non-blocking async PBO read.
   */
  private requestPixelRead(): void {
    if (!this.gl || !this.pbo) return;

    const gl = this.gl;
    if (this.currentSync) {
      gl.deleteSync(this.currentSync);
      this.currentSync = null;
    }

    gl.bindBuffer(gl.PIXEL_PACK_BUFFER, this.pbo);
    // Non-blocking asynchronous dispatch
    gl.readPixels(this.lastCursorX, this.lastCursorY, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, 0);
    gl.bindBuffer(gl.PIXEL_PACK_BUFFER, null);

    this.currentSync = gl.fenceSync(gl.SYNC_GPU_COMMANDS_COMPLETE, 0);
    this.isReadPending = true;
  }

  /**
   * Called every frame in animation loop to poll fence status without stalling CPU.
   */
  pollFence(): void {
    if (!this.gl || !this.pbo || !this.currentSync || !this.isReadPending) return;

    const gl = this.gl;
    const status = gl.clientWaitSync(this.currentSync, 0, 0);

    if (status === gl.ALREADY_SIGNALED || status === gl.CONDITION_SATISFIED) {
      gl.bindBuffer(gl.PIXEL_PACK_BUFFER, this.pbo);
      gl.getBufferSubData(gl.PIXEL_PACK_BUFFER, 0, this.pixelBuffer);
      gl.bindBuffer(gl.PIXEL_PACK_BUFFER, null);

      gl.deleteSync(this.currentSync);
      this.currentSync = null;
      this.isReadPending = false;

      const alpha = this.pixelBuffer[3]! / 255.0;
      const over = alpha > this.alphaThreshold;

      if (over !== this.isOverCharacter) {
        this.isOverCharacter = over;
        if (this.onHitTestChange) {
          this.onHitTestChange(over);
        }
      }
    }
  }

  /**
   * Synchronous fallback for test environments or immediate queries.
   */
  readPixelSync(
    canvasX: number,
    canvasY: number,
    canvasHeight: number,
  ): { r: number; g: number; b: number; a: number } {
    if (!this.gl) return { r: 0, g: 0, b: 0, a: 0 };
    const gl = this.gl;
    const glY = Math.floor(canvasHeight - canvasY);
    const pixel = new Uint8Array(4);
    gl.readPixels(Math.floor(canvasX), glY, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixel);
    return {
      r: pixel[0]!,
      g: pixel[1]!,
      b: pixel[2]!,
      a: pixel[3]! / 255.0,
    };
  }

  getOverCharacter(): boolean {
    return this.isOverCharacter;
  }

  dispose(): void {
    if (this.gl) {
      if (this.currentSync) {
        this.gl.deleteSync(this.currentSync);
        this.currentSync = null;
      }
      if (this.pbo) {
        this.gl.deleteBuffer(this.pbo);
      }
    }
  }
}
