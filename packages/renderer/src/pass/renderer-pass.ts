import { WebGPURenderer } from "three/webgpu";

/**
 * One stage of the frame, timed on the GPU under its name.
 */
export interface IRendererPass {
  /** The name the frame report states the pass under. */
  readonly name: string;
  /**
   * @param renderer - The renderer the frame draws with.
   */
  render(renderer: WebGPURenderer): void;
  /** Releases whatever the pass holds on the device. */
  dispose(): void;
}
