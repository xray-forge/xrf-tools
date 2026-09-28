import { TRendererColor } from "#/contract/renderer-color";
import { IRendererLighting } from "#/contract/renderer-lighting";
import { DEFAULT_RENDERER_LIGHTING } from "#/lighting/default-lighting";

/**
 * The engine's noon, every colour turned to the grey of its own luminance: what an asset viewer lights with, so a
 * texture shows its own colours as brightly as noon lights them.
 */
export const NEUTRAL_RENDERER_LIGHTING: IRendererLighting = {
  ...DEFAULT_RENDERER_LIGHTING,
  ambientColor: toGrey(DEFAULT_RENDERER_LIGHTING.ambientColor),
  hemisphereColor: toGrey(DEFAULT_RENDERER_LIGHTING.hemisphereColor),
  skyIrradiance: toGrey(DEFAULT_RENDERER_LIGHTING.skyIrradiance),
  sunColor: toGrey(DEFAULT_RENDERER_LIGHTING.sunColor),
};

/** Rec. 709 luma of the values as stored. */
function toGrey(color: TRendererColor): TRendererColor {
  const luma: number = 0.2126 * color[0] + 0.7152 * color[1] + 0.0722 * color[2];

  return [luma, luma, luma];
}
