import { RenderTarget, WebGPURenderer } from "three/webgpu";

import { IRendererFrameSize } from "#/pass/renderer-frame-size";
import { IRendererPass } from "#/pass/renderer-pass";

/**
 * A pass resolving the jittered frames with their history at the output's size: TAA's, or FSR 2's. The frame's jitter
 * is the graph's; the resolve reads where this frame's samples stand from the frame.
 */
export interface ITemporalUpscaler extends IRendererPass {
  /** The frame as resolved, with the drawn depth at the output's size for the helpers to test against. */
  readonly output: RenderTarget;
  /** Passes it needs drawn before the blended surfaces are. */
  readonly beforeBlended: ReadonlyArray<IRendererPass>;
  resize(renderer: WebGPURenderer, size: IRendererFrameSize): void;
  /** Forgets the history, for a view that jumped. */
  resetHistory(): void;
}
