import { CanvasTarget, LinearSRGBColorSpace, NoToneMapping, WebGPURenderer } from "three/webgpu";

import { IRendererDevice } from "#/contract/renderer-device";
import { RendererDeviceFailure } from "#/device/renderer-device-failure";
import { getRendererBackend, IRendererBackend } from "#/internals/renderer-backend";
import { RendererPassInspector } from "#/timing/renderer-pass-inspector";

/**
 * One GPU device, as three drives it for the frame: drawing on whichever canvas target is current.
 */
export class RendererDevice {
  /**
   * @returns The device, up.
   * @throws {RendererDeviceFailure} Where there is no WebGPU device to draw with.
   */
  public static async open(): Promise<RendererDevice> {
    const renderer: WebGPURenderer = new WebGPURenderer({
      alpha: true,
      antialias: false,
      // Drawn on by nothing: a view brings its own canvas, and textures and captures need none.
      canvas: new OffscreenCanvas(1, 1),
      trackTimestamp: true,
    });

    try {
      await renderer.init();
    } catch (error) {
      renderer.dispose();

      throw new RendererDeviceFailure(`The GPU device could not be created: ${error}`);
    }

    if (!getRendererBackend(renderer).isWebGPUBackend) {
      renderer.dispose();

      throw new RendererDeviceFailure("WebGPU is unavailable, and this renderer has no fallback.");
    }

    return new RendererDevice(renderer);
  }

  public readonly renderer: WebGPURenderer;
  /** What tags every render with the pass issuing it, for its GPU time. */
  public readonly inspector: RendererPassInspector = new RendererPassInspector();
  /** The canvas three was made with, drawn on by nothing, so a detached view leaves the device a target. */
  public readonly headless: CanvasTarget;
  /** Whether the device can time passes. */
  public readonly isGpuTimed: boolean;

  private constructor(renderer: WebGPURenderer) {
    this.renderer = renderer;
    this.headless = renderer.getCanvasTarget();
    this.isGpuTimed = renderer.hasFeature("timestamp-query");

    // A frame is several renders, so its counters reset once per frame rather than once per render.
    renderer.info.autoReset = false;
    renderer.inspector = this.inspector;
    // Every pass clears what it means to; three clearing before each render would erase the G-buffer.
    renderer.autoClear = false;
    // The frame is already the bytes the canvas shows: raw values, tonemapped by the combine pass.
    renderer.outputColorSpace = LinearSRGBColorSpace;
    renderer.toneMapping = NoToneMapping;
  }

  /** The GPU, as far as the browser tells. */
  public describe(): IRendererDevice {
    const backend: IRendererBackend = getRendererBackend(this.renderer);

    return {
      architecture: backend.device?.adapterInfo?.architecture ?? "",
      features: [...(backend.device?.features ?? [])].sort(),
      vendor: backend.device?.adapterInfo?.vendor ?? "",
    };
  }

  public dispose(): void {
    this.renderer.dispose();
  }
}
