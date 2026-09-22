import { IRendererFrame } from "#/graph/renderer-frame";

/**
 * One stage of the frame, timed on the GPU under its name.
 */
export interface IRendererPass {
  /** The name the frame report states the pass under. */
  readonly name: string;
  /**
   * @param frame - What the frame draws with.
   */
  render(frame: IRendererFrame): void;
  /** Releases whatever the pass holds on the device. */
  dispose(): void;
}
