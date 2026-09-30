import { ComputeNode } from "three/webgpu";

import { FullScreenDraw } from "#/pass/full-screen-draw";

/**
 * Where a pass names the pipelines it draws the next frame with, for the compile lane to build them off the frame
 * before the frame draws them. Named again before every frame: a draw or kernel named before is known.
 */
export interface IRendererPipelines {
  /**
   * @param draw - A full screen draw, compiled for its target.
   */
  draw(draw: FullScreenDraw): void;
  /**
   * @param kernels - Compute kernels, compiled as three dispatches them.
   */
  compute(kernels: ReadonlyArray<ComputeNode>): void;
}
