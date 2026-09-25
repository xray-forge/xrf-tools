import {
  DEFAULT_RENDERER_GRASS_SETTINGS,
  ERendererAmbientOcclusionQuality,
  ERendererAntialiasing,
} from "@xrf/renderer";

/** The engine's three cascade widths (`render_phase_sun.cpp`), and a fourth reaching three times as far. */
export const RENDER_SHADOW_CASCADE_WIDTHS: ReadonlyArray<number> = [20, 40, 160, 480];

/** The map resolutions offered, in texels across. */
export const RENDER_SHADOW_RESOLUTIONS: ReadonlyArray<number> = [1024, 2048, 4096];

/** The bounds each shadow value is offered between. */
export const RENDER_SHADOW_LIMITS = {
  bias: { max: 5, min: 0, step: 0.25 },
  blend: { max: 0.2, min: 0, step: 0.01 },
  filter: { max: 3, min: 0, step: 1 },
  reach: { max: 1000, min: 100, step: 50 },
} as const;

/** The engine's grass density, `r__detail_density`, which is a spacing: a smaller one plants more. */
const GAME_GRASS_DENSITY: number = DEFAULT_RENDERER_GRASS_SETTINGS.density;

/**
 * The bounds each grass value is offered between: density as how many times the game's the tufts stand, up to the
 * engine's densest of 0.1, and the engine's own console ranges for the rest.
 */
export const RENDER_GRASS_LIMITS = {
  density: { max: 6, min: 0.65, step: 0.05 },
  height: { max: 2, min: 0.5, step: 0.1 },
  radius: { max: 150, min: 49, step: 1 },
} as const;

/**
 * @param density - The engine's density, a spacing between tufts.
 * @returns How many times the game's density that is, as the settings offer it: higher is denser.
 */
export function toGrassDensityScale(density: number): number {
  return GAME_GRASS_DENSITY / density;
}

/**
 * @param scale - How many times the game's density the settings ask for.
 * @returns The engine's density that is.
 */
export function fromGrassDensityScale(scale: number): number {
  return GAME_GRASS_DENSITY / scale;
}

/** The bounds each ambient occlusion value is offered between. */
export const RENDER_AMBIENT_OCCLUSION_LIMITS = {
  radius: { max: 4, min: 0.25, step: 0.25 },
  strength: { max: 2, min: 0, step: 0.1 },
} as const;

/**
 * @param quality - An ambient occlusion quality.
 * @returns Its name as the settings say it.
 */
export function describeRenderAmbientOcclusionQuality(quality: ERendererAmbientOcclusionQuality): string {
  switch (quality) {
    case ERendererAmbientOcclusionQuality.LOW:
      return "Low";
    case ERendererAmbientOcclusionQuality.MEDIUM:
      return "Medium";
    case ERendererAmbientOcclusionQuality.HIGH:
      return "High";
    case ERendererAmbientOcclusionQuality.ULTRA:
      return "Ultra";
  }
}

/**
 * @param mode - An antialiasing mode.
 * @returns Its name as the settings say it.
 */
export function describeRenderAntialiasing(mode: ERendererAntialiasing): string {
  switch (mode) {
    case ERendererAntialiasing.NONE:
      return "None";
    case ERendererAntialiasing.FXAA:
      return "FXAA";
    case ERendererAntialiasing.SMAA:
      return "SMAA";
    case ERendererAntialiasing.TAA:
      return "TAA";
  }
}

/**
 * @param blend - How far in from a cascade's edge the next is mixed in, as a share of its width.
 * @returns It as the settings read it: a percentage, or the engine's hard switch for none.
 */
export function formatCascadeBlend(blend: number): string {
  return blend > 0 ? `${Math.round(blend * 100)}%` : "Hard";
}
