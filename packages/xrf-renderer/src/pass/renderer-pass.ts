import { WebGPURenderer } from "three/webgpu";

import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererFrameSize } from "#/sampling/renderer-frame-size";

/**
 * One stage of the frame, timed on the GPU under its name.
 */
export interface IRendererPass {
  /** The name the frame report states the pass under. */
  readonly name: string;
  /**
   * Sizes what the pass holds to the frame and readies it on the renderer: whenever either changes, and as it joins.
   *
   * @param renderer - The renderer the pass draws with.
   * @param size - The frame's size.
   */
  resize?(renderer: WebGPURenderer, size: IRendererFrameSize): void;
  /**
   * @param frame - What the frame draws with.
   */
  render(frame: IRendererFrame): void;
  /** Releases whatever the pass holds on the device. */
  dispose(): void;
}
