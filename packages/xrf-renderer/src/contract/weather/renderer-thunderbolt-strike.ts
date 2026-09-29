import { TRendererVector } from "#/contract/renderer-vector";
import { IRendererThunderboltGlow } from "#/contract/weather/renderer-thunderbolt-glow";

/**
 * A bolt striking this frame, as `dxThunderboltRender` draws it.
 */
export interface IRendererThunderboltStrike {
  /** The bolt, a key of the thunder's bolts: its model, and its glows' textures. */
  bolt: string;
  /**
   * `current_xform` in renderer space: where each of the model's own axes, as its mesh lies in renderer space, points,
   * scaled by the bolt's length.
   */
  axes: readonly [TRendererVector, TRendererVector, TRendererVector];
  /** Where it strikes from, its top, in renderer space. */
  position: TRendererVector;
  /** What the model's texture coordinates are shifted down by, flickering between its halves late in the strike. */
  shift: number;
  top: IRendererThunderboltGlow;
  center: IRendererThunderboltGlow;
}
