import { IRendererLighting, TRendererColor } from "#/contract/renderer-lighting";
import { DEFAULT_RENDERER_LIGHTING } from "#/lighting/default-lighting";

/**
 * A lighting value with every colour turned to the grey of its own luminance: as bright as it was, and neutral.
 *
 * @param lighting - The lighting to neutralise.
 * @returns The same lighting, uncoloured.
 */
export function toNeutralRendererLighting(lighting: IRendererLighting): IRendererLighting {
  return {
    ...lighting,
    ambientColor: toGrey(lighting.ambientColor),
    fog: lighting.fog ? { ...lighting.fog, color: toGrey(lighting.fog.color) } : null,
    hemisphereColor: toGrey(lighting.hemisphereColor),
    skyIrradiance: toGrey(lighting.skyIrradiance),
    sunColor: toGrey(lighting.sunColor),
  };
}

/**
 * The engine's noon, uncoloured: what an asset viewer lights with, so a texture shows its own colours.
 */
export const NEUTRAL_RENDERER_LIGHTING: IRendererLighting = toNeutralRendererLighting(DEFAULT_RENDERER_LIGHTING);

/** Rec. 709 luma of the values as stored. */
function toGrey(color: TRendererColor): TRendererColor {
  const luma: number = 0.2126 * color[0] + 0.7152 * color[1] + 0.0722 * color[2];

  return [luma, luma, luma];
}
