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

/** Cascades the sun's shadow can be cut into at most. */
export const RENDERER_MAX_SHADOW_CASCADES: number = 4;

/**
 * The sun's shadow: cascades of maps, each a square of the level seen from the sun, drawn every frame through the
 * static draws and sampled by the sun's light. The engine's are three, 20, 40 and 160 metres across, at 2048 texels
 * (`render_phase_sun.cpp`, `r2_smap_size`).
 */
export interface IRendererShadowSettings {
  isEnabled: boolean;
  /** Each cascade's width in metres, nearest first; as many cascades as widths, at most `RENDERER_MAX_SHADOW_CASCADES`. */
  cascades: ReadonlyArray<number>;
  /** Texels each cascade's map is across. */
  resolution: number;
  /** Texels the filter reaches from the one sampled, each way: zero for one comparison, one for a three by three. */
  filter: number;
  /**
   * How far a point is moved along its normal before it is compared, in texels of its cascade: keeps a lit surface
   *  from shadowing itself.
   */
  bias: number;
  /** Metres towards the sun past a cascade that its casters may stand: a tower outside the map still shades into it. */
  reach: number;
  /**
   * How far in from a cascade's edge, as a share of its width, the next cascade is mixed in: none switches maps at a
   * line, as the engine does.
   */
  blend: number;
  /**
   * Whether cascade `n` is drawn at most every `2^n` frames, the far ones sharing frames the near one does not. A map
   * holds depth in the world and is sampled with the matrix it was drawn with, so a map a frame or three old is exact
   * for everything that stands still; only something moving would cast late.
   */
  isStaggered: boolean;
}

/** The engine's own cascades, and a filter a texel wide. */
export const DEFAULT_RENDERER_SHADOW_SETTINGS: IRendererShadowSettings = {
  bias: 1.5,
  blend: 0.1,
  cascades: [20, 40, 160],
  filter: 1,
  isEnabled: true,
  isStaggered: true,
  reach: 400,
  resolution: 2048,
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
  shadows: IRendererShadowSettings;
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
    shadows: DEFAULT_RENDERER_SHADOW_SETTINGS,
  },
  [ERendererPreset.EDITING]: {
    antialiasing: ERendererAntialiasing.NONE,
    isGpuTimed: true,
    lod: DEFAULT_RENDERER_LOD_SETTINGS,
    shadows: { ...DEFAULT_RENDERER_SHADOW_SETTINGS, isEnabled: false },
  },
};

/** What was changed on top of a preset, feature by feature. */
export interface IRendererFeatureOverrides {
  antialiasing?: ERendererAntialiasing;
  isGpuTimed?: boolean;
  lod?: Partial<IRendererLodSettings>;
  shadows?: Partial<IRendererShadowSettings>;
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

  const shadows: Partial<IRendererShadowSettings> = toShadowOverrides(source.shadows);

  if (Object.keys(shadows).length) {
    choice.overrides.shadows = shadows;
  }

  return choice;
}

/**
 * @param choice - A preset and what was changed on top of it.
 * @returns Every feature as the choice sets it.
 */
export function resolveRendererFeatures(choice: IRendererFeatureChoice): IRendererFeatureSettings {
  const preset: IRendererFeatureSettings = RENDERER_PRESETS[choice.preset];
  const { antialiasing, isGpuTimed, lod, shadows } = choice.overrides;

  return {
    antialiasing: antialiasing ?? preset.antialiasing,
    isGpuTimed: isGpuTimed ?? preset.isGpuTimed,
    lod: { ...preset.lod, ...lod },
    shadows: { ...preset.shadows, ...shadows },
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
    (Object.keys(preset.lod) as Array<keyof IRendererLodSettings>).some(
      (key) => resolved.lod[key] !== preset.lod[key]
    ) ||
    (Object.keys(preset.shadows) as Array<keyof IRendererShadowSettings>).some((key) =>
      key === "cascades"
        ? resolved.shadows.cascades.join() !== preset.shadows.cascades.join()
        : resolved.shadows[key] !== preset.shadows[key]
    )
  );
}

/**
 * @param stored - What was stored for the shadow overrides.
 * @returns The ones the shadows take: finite numbers, a flag, and a run of positive widths within the cascade limit.
 */
function toShadowOverrides(stored: unknown): Partial<IRendererShadowSettings> {
  const source: Record<string, unknown> = stored && typeof stored === "object" ? (stored as never) : {};
  const overrides: Partial<IRendererShadowSettings> = {};

  for (const key of ["isEnabled", "isStaggered"] as const) {
    if (typeof source[key] === "boolean") {
      overrides[key] = source[key];
    }
  }

  for (const key of ["bias", "blend", "filter", "reach", "resolution"] as const) {
    const value: unknown = source[key];

    if (typeof value === "number" && Number.isFinite(value) && value >= 0) {
      overrides[key] = value;
    }
  }

  const cascades: unknown = source.cascades;

  if (
    Array.isArray(cascades) &&
    cascades.length > 0 &&
    cascades.length <= RENDERER_MAX_SHADOW_CASCADES &&
    cascades.every((width: unknown) => typeof width === "number" && Number.isFinite(width) && width > 0)
  ) {
    overrides.cascades = cascades as Array<number>;
  }

  return overrides;
}
