import { ERendererAmbientOcclusionQuality, ERendererAntialiasing } from "@xrf/renderer";

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
