/**
 * How the finished frame's edges are smoothed. Deferred shading rules hardware multisampling out, so every mode is a
 * pass over the frame; temporal AA comes with `Enhanced`.
 */
export enum ERendererAntialiasing {
  NONE = "none",
  /** One pass, softest, cheapest. */
  FXAA = "fxaa",
  /** Three passes over the frame's edges, crisp and stable, `Base`'s choice. */
  SMAA = "smaa",
}

/**
 * The engine's switches between a clump of trees and its impostor, and between a progressive mesh's windows, on a
 * screen area: a sphere's radius over its squared distance, against thresholds that scale with the drawing's size and
 * field of view (`r2_R_calculate.cpp`).
 */
export interface IRendererLodSettings {
  /** Whether impostors draw at all; off, every tree draws in full at every distance. */
  isImpostors: boolean;
  /** `r2_ssa_lod_a`: below it the impostor draws. */
  ssaA: number;
  /** `r2_ssa_lod_b`: above it the trees draw; between the two, both. */
  ssaB: number;
  /** `r__ssa_discard`: below it neither draws. */
  ssaDiscard: number;
  /** `r__geometry_lod`: what the drawing's area is scaled by before the thresholds are taken from it. */
  geometryLod: number;
  /** `r__ssa_glod_start`: above it a progressive mesh draws its whole detail. */
  ssaGlodStart: number;
  /** `r__ssa_glod_end`: below it a progressive mesh draws its coarsest window. */
  ssaGlodEnd: number;
}

/** The engine's own values (`xrRender_console.cpp`). */
export const DEFAULT_RENDERER_LOD_SETTINGS: IRendererLodSettings = {
  geometryLod: 0.75,
  isImpostors: true,
  ssaA: 64,
  ssaB: 48,
  ssaDiscard: 3.5,
  ssaGlodEnd: 64,
  ssaGlodStart: 256,
};

/**
 * What the renderer's features are set to: the same for every consumer, chosen as a preset and whatever was changed
 * on top of it. A feature that is off costs nothing: its passes leave the frame and its targets are freed.
 */
export interface IRendererFeatureSettings {
  antialiasing: ERendererAntialiasing;
  /** Whether every pass is timed on the GPU for the report. */
  isGpuTimed: boolean;
  lod: IRendererLodSettings;
}

/**
 * The named sets of features: `Base` the engine's defaults, `Editing` responsiveness before looks for an editor.
 */
export enum ERendererPreset {
  BASE = "base",
  EDITING = "editing",
}

/** What each preset sets every feature to. */
export const RENDERER_PRESETS: Readonly<Record<ERendererPreset, IRendererFeatureSettings>> = {
  [ERendererPreset.BASE]: {
    antialiasing: ERendererAntialiasing.SMAA,
    isGpuTimed: true,
    lod: DEFAULT_RENDERER_LOD_SETTINGS,
  },
  [ERendererPreset.EDITING]: {
    antialiasing: ERendererAntialiasing.NONE,
    isGpuTimed: true,
    lod: DEFAULT_RENDERER_LOD_SETTINGS,
  },
};

/** What was changed on top of a preset, feature by feature. */
export interface IRendererFeatureOverrides {
  antialiasing?: ERendererAntialiasing;
  isGpuTimed?: boolean;
  lod?: Partial<IRendererLodSettings>;
}

/** A preset and what was changed on top of it, which is what a consumer stores. */
export interface IRendererFeatureChoice {
  preset: ERendererPreset;
  overrides: IRendererFeatureOverrides;
}

export const DEFAULT_RENDERER_FEATURE_CHOICE: IRendererFeatureChoice = {
  overrides: {},
  preset: ERendererPreset.BASE,
};

/**
 * @param stored - What a consumer stored for its choice, parsed from wherever it keeps it.
 * @returns The choice, with every value that is not one the features take dropped; the default for nothing usable.
 */
export function toRendererFeatureChoice(stored: unknown): IRendererFeatureChoice {
  if (!stored || typeof stored !== "object") {
    return DEFAULT_RENDERER_FEATURE_CHOICE;
  }

  const { preset, overrides } = stored as Partial<Record<keyof IRendererFeatureChoice, unknown>>;
  const source: Record<string, unknown> = overrides && typeof overrides === "object" ? (overrides as never) : {};
  const lod: Record<string, unknown> = source.lod && typeof source.lod === "object" ? (source.lod as never) : {};
  const choice: IRendererFeatureChoice = {
    overrides: {},
    preset: Object.values(ERendererPreset).find((it) => it === preset) ?? DEFAULT_RENDERER_FEATURE_CHOICE.preset,
  };
  const antialiasing: ERendererAntialiasing | undefined = Object.values(ERendererAntialiasing).find(
    (it) => it === source.antialiasing
  );
  const lodOverrides: Partial<IRendererLodSettings> = {};

  for (const key of Object.keys(DEFAULT_RENDERER_LOD_SETTINGS) as Array<keyof IRendererLodSettings>) {
    const value: unknown = lod[key];

    if (
      typeof value === typeof DEFAULT_RENDERER_LOD_SETTINGS[key] &&
      (typeof value !== "number" || Number.isFinite(value))
    ) {
      (lodOverrides as Record<string, unknown>)[key] = value;
    }
  }

  if (antialiasing) {
    choice.overrides.antialiasing = antialiasing;
  }

  if (typeof source.isGpuTimed === "boolean") {
    choice.overrides.isGpuTimed = source.isGpuTimed;
  }

  if (Object.keys(lodOverrides).length) {
    choice.overrides.lod = lodOverrides;
  }

  return choice;
}

/**
 * @param choice - A preset and what was changed on top of it.
 * @returns Every feature as the choice sets it.
 */
export function resolveRendererFeatures(choice: IRendererFeatureChoice): IRendererFeatureSettings {
  const preset: IRendererFeatureSettings = RENDERER_PRESETS[choice.preset];
  const { antialiasing, isGpuTimed, lod } = choice.overrides;

  return {
    antialiasing: antialiasing ?? preset.antialiasing,
    isGpuTimed: isGpuTimed ?? preset.isGpuTimed,
    lod: { ...preset.lod, ...lod },
  };
}

/**
 * @param choice - A preset and what was changed on top of it.
 * @returns Whether the features differ from the preset's, which a settings view shows as custom.
 */
export function isRendererFeatureChoiceCustom(choice: IRendererFeatureChoice): boolean {
  const resolved: IRendererFeatureSettings = resolveRendererFeatures(choice);
  const preset: IRendererFeatureSettings = RENDERER_PRESETS[choice.preset];

  return (
    resolved.antialiasing !== preset.antialiasing ||
    resolved.isGpuTimed !== preset.isGpuTimed ||
    (Object.keys(preset.lod) as Array<keyof IRendererLodSettings>).some((key) => resolved.lod[key] !== preset.lod[key])
  );
}
