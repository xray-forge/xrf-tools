import { DEFAULT_RENDERER_LOD_SETTINGS, IRendererLodSettings } from "@xrf/renderer";

/**
 * How far the viewer draws a clump of trees in full before its impostor takes over.
 */
export interface ILevelLodOptions {
  /** The distance a clump switches at, against the engine's own: one at `r__geometry_lod` 0.75. */
  distance: number;
}

export const DEFAULT_LEVEL_LOD_OPTIONS: ILevelLodOptions = {
  distance: 1,
};

/** The bounds each value is offered between. */
export const LEVEL_LOD_LIMITS = {
  /** A twentieth of the engine's distance, where a clump turns at arm's length, to three times it, past any fog. */
  distance: { max: 3, min: 0.05, step: 0.05 },
} as const;

/**
 * @param lod - The distance the toolbar asks for.
 * @param isImpostors - Whether the toolbar draws impostors at all.
 * @returns The renderer's LOD settings: the engine's thresholds, with the detail scale the distance comes to.
 */
export function toLevelRendererLod(lod: ILevelLodOptions, isImpostors: boolean): IRendererLodSettings {
  return {
    ...DEFAULT_RENDERER_LOD_SETTINGS,
    // A clump's screen area falls with the square of its distance, and every threshold with the detail scale.
    geometryLod: DEFAULT_RENDERER_LOD_SETTINGS.geometryLod * lod.distance ** 2,
    isImpostors,
  };
}
