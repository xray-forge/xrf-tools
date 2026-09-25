import { PerspectiveCamera, RenderTarget, WebGPURenderer } from "three/webgpu";

import { IRendererPass } from "#/pass/renderer-pass";

/**
 * A pass resolving jittered frames with their history at the output's size: TAA's, or FSR 2's. It jitters the camera
 * before anything draws and gives it back before the helpers do.
 */
export interface ITemporalUpscaler extends IRendererPass {
  /** The frame as resolved, with the drawn depth at the output's size for the helpers to test against. */
  readonly output: RenderTarget;
  /**
   * Sizes its targets to the output and its jitter to the drawing, either change starting its history afresh.
   *
   * @param renderer - The renderer the targets are drawn by.
   * @param width - The output's width, in device pixels.
   * @param height - And its height.
   * @param renderWidth - The scene's width as drawn.
   * @param renderHeight - And its height.
   */
  resize(renderer: WebGPURenderer, width: number, height: number, renderWidth: number, renderHeight: number): void;
  /**
   * @param camera - The drawing camera, its projection as its controller left it, offset by this frame's jitter.
   */
  jitter(camera: PerspectiveCamera): void;
}
