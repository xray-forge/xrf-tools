import { ERendererAntialiasing } from "@xrf/renderer";

/** The engine's three cascade widths (`render_phase_sun.cpp`), and a fourth reaching three times as far. */
export const RENDER_SHADOW_CASCADE_WIDTHS: ReadonlyArray<number> = [20, 40, 160, 480];

/** The map resolutions offered, in texels across. */
export const RENDER_SHADOW_RESOLUTIONS: ReadonlyArray<number> = [1024, 2048, 4096];

/** The bounds each shadow value is offered between. */
export const RENDER_SHADOW_LIMITS = {
  bias: { max: 5, min: 0, step: 0.25 },
  filter: { max: 3, min: 0, step: 1 },
  reach: { max: 1000, min: 100, step: 50 },
} as const;

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
  }
}
