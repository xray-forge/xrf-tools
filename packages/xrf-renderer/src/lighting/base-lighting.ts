import { EPS } from "@xrf/math";

import { TRendererColor } from "#/contract/renderer-color";
import { IRendererLighting } from "#/contract/renderer-lighting";
import { TRendererVector } from "#/contract/renderer-vector";
import { IBaseLightingConstants } from "#/lighting/base-lighting-constants";

/** `ps_r2_gloss_factor` (`Layers/xrRender/xrRender_console.cpp`). */
const GLOSS_FACTOR: number = 4;

/** The floor `phase_combine` keeps ambient above. */
const MINIMUM_AMBIENT: number = 0.001;

/**
 * The constants the engine would bind for this lighting.
 *
 * @param lighting - The lighting value a consumer sent.
 * @returns What the sun, hemisphere, combine and fog passes read.
 */
export function toBaseLightingConstants(lighting: IRendererLighting): IBaseLightingConstants {
  const [fogOffset, fogScale] = lighting.fog ? toFogParams(lighting.fog.distance, lighting.fog.density) : [0, 0];

  return {
    ambient: scale(lighting.ambientColor, (channel: number) => Math.max(channel * 2, MINIMUM_AMBIENT)),
    // `hemi_color * 2 + EPS` in `CEnvDescriptorMixer::lerp`, so a black hemisphere is never exactly zero, then doubled
    // again by `phase_combine`.
    environment: scale(lighting.hemisphereColor, (channel: number) => (channel * 2 + EPS) * 2),
    fogColor: lighting.fog?.color ?? [0, 0, 0],
    fogOffset,
    fogScale,
    isFogged: lighting.fog !== null,
    rainDensity: lighting.rain?.density ?? 0,
    skyIrradiance: lighting.skyIrradiance,
    sunColor: lighting.sunColor,
    sunDirection: normalise(lighting.sunDirection),
    sunSpecular: toSunSpecular(lighting.sunColor),
  };
}

/**
 * `u_diffuse2s`: a light's specular weight from its colour (`Layers/xrRender_R2/r2_types.h`).
 *
 * @param color - The light's colour.
 * @returns Its specular weight.
 */
export function toSunSpecular(color: TRendererColor): number {
  const mean: number = (color[0] + color[1] + color[2]) / 3;

  return GLOSS_FACTOR * (mean < 1 ? Math.pow(mean, 2 / 3) : mean);
}

/**
 * `fog_params` from a keyframe's fog: offset and scale of the ramp from where fog starts to where it is total.
 *
 * @param distance - `fog_distance`.
 * @param density - `fog_density`.
 * @returns `x` and `w` of `fog_params`.
 */
function toFogParams(distance: number, density: number): [number, number] {
  // `CEnvDescriptorMixer::lerp` and `cl_fog_params` (`xrEngine/Environment_misc.cpp`, `Blender_Recorder_StandartBinding`).
  const near: number = (1 - density) * 0.85 * distance;
  const far: number = 0.99 * distance;
  const range: number = 1 / (far - near);

  return [-near * range, range];
}

function scale(color: TRendererColor, by: (channel: number) => number): TRendererColor {
  return [by(color[0]), by(color[1]), by(color[2])];
}

function normalise(vector: TRendererVector): TRendererVector {
  const length: number = Math.hypot(vector[0], vector[1], vector[2]) || 1;

  return [vector[0] / length, vector[1] / length, vector[2] / length];
}
