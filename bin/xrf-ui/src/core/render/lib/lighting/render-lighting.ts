import { RenderAssetLighting } from "@/core/ipc/types/xrf-renderer";
import { toRawColor } from "@/core/render/lib/scene/render-color";

/**
 * How a preview is lit, which is the viewer's own answer rather than anything an X-Ray file carries.
 */
export interface IRenderLighting {
  /** Degrees above the horizon the light sits at, `90` being directly overhead. */
  sunElevation: number;
  /** Degrees around the vertical axis, `0` looking along the scene's own `+z`. */
  sunAzimuth: number;
  sunIntensity: number;
  /** Hex colour of the directional light. */
  sunColor: number;
  /** Uniform light standing in for everything the one direction does not reach. */
  ambientIntensity: number;
  ambientColor: number;
}

/** The bounds each value is offered between, so every surface adjusting lighting offers the same range. */
export const RENDER_LIGHTING_LIMITS = {
  ambientIntensity: { max: 4, min: 0, step: 0.05 },
  sunAzimuth: { max: 180, min: -180, step: 1 },
  sunElevation: { max: 90, min: -15, step: 1 },
  sunIntensity: { max: 6, min: 0, step: 0.05 },
} as const;

/**
 * @param lighting - An asset viewer's light, as its controls set it.
 * @returns The same, as a native viewport lights an asset in place of a weather.
 */
export function toNativeAssetLighting(lighting: IRenderLighting): RenderAssetLighting {
  return {
    ambientColor: toRawColor(lighting.ambientColor),
    ambientIntensity: lighting.ambientIntensity,
    sunAzimuth: lighting.sunAzimuth,
    sunColor: toRawColor(lighting.sunColor),
    sunElevation: lighting.sunElevation,
    sunIntensity: lighting.sunIntensity,
  };
}
