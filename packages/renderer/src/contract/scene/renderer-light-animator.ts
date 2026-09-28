import { TRendererColor } from "#/contract/renderer-color";

/**
 * A colour animation (`CLAItem`): keys a frame apart at its rate, the colour between two keys blended by the frame.
 */
export interface IRendererLightAnimator {
  fps: number;
  frameCount: number;
  /** Each key's frame, ascending, the first at zero. */
  frames: ReadonlyArray<number>;
  /** Each key's colour, each channel in `[0, 255]`. */
  colors: ReadonlyArray<TRendererColor>;
}
