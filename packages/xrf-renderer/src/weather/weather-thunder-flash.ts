import { TRendererColor } from "#/contract/renderer-color";
import { TRendererVector } from "#/contract/renderer-vector";
import { IRendererThunderboltStrike } from "#/contract/weather/renderer-thunderbolt-strike";

/**
 * What a strike does to a frame: the colour it lights the sky, the sun and the fog with, the way its light travels,
 * and the bolt drawn.
 */
export interface IWeatherThunderFlash {
  /** The bolt's `color_anim` now, each channel in `[0, 1]`. */
  color: TRendererColor;
  /** From the bolt towards the view, which the sun is turned to while it strikes, in engine space. */
  direction: TRendererVector;
  strike: IRendererThunderboltStrike;
}
