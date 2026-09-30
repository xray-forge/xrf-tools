import { WebGPURenderer } from "three/webgpu";

import { IRendererSettings } from "#/contract/renderer-settings";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
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
   * Names every pipeline of its own the pass draws the next frame with, making first what changed for it since: the
   * frame is drawn once all of them compiled, so none is built as the frame draws. Called before every frame, after
   * the pass was sized for it.
   *
   * @param pipelines - Where they are named.
   * @param settings - What the frame is drawn with.
   */
  listPipelines?(pipelines: IRendererPipelines, settings: IRendererSettings): void;
  /**
   * @param frame - What the frame draws with.
   */
  render(frame: IRendererFrame): void;
  /** Releases whatever the pass holds on the device. */
  dispose(): void;
}
