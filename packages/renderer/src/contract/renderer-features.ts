/**
 * How the finished frame's edges are smoothed. Deferred shading rules hardware multisampling out, so every mode is a
 * pass over the frame.
 */
export enum ERendererAntialiasing {
  NONE = "none",
  /** One pass, softest, cheapest. */
  FXAA = "fxaa",
  /** Three passes over the frame's edges, crisp and stable, `Base`'s choice. */
  SMAA = "smaa",
  /**
   * Temporal: every frame's samples jittered within the pixel and resolved with the frames before, found by the motion
   * every surface writes. Smooths inside a surface too, cut-out foliage and thin wires, where SMAA finds no edge.
   */
  TAA = "taa",
  /**
   * FSR 2: AMD's temporal upscaler, from the same jittered frames, motion and depth as TAA, with locks that keep thin
   * features and a reactive mask from what the blended surfaces changed.
   */
  FSR2 = "fsr2",
}

/**
 * @param mode - An antialiasing mode.
 * @returns Whether it resolves jittered frames with their history, which jitters every scene pass.
 */
export function isRendererTemporal(mode: ERendererAntialiasing): boolean {
  return mode === ERendererAntialiasing.TAA || mode === ERendererAntialiasing.FSR2;
}

/**
 * How much smaller than the output the scene is drawn and then upscaled: FSR's quality modes, by the ratio of the
 * output's side to the drawing's. TAA upscales in its resolve, FSR in its own, and the other modes with FSR 1.
 */
export enum ERendererRenderScale {
  NATIVE = "native",
  /** 1.5: two thirds of each side. */
  QUALITY = "quality",
  /** 1.7. */
  BALANCED = "balanced",
  /** 2: half of each side. */
  PERFORMANCE = "performance",
}

/** Each render scale's ratio of the output's side to the drawing's. */
export const RENDERER_RENDER_SCALE_RATIOS: Readonly<Record<ERendererRenderScale, number>> = {
  [ERendererRenderScale.NATIVE]: 1,
  [ERendererRenderScale.QUALITY]: 1.5,
  [ERendererRenderScale.BALANCED]: 1.7,
  [ERendererRenderScale.PERFORMANCE]: 2,
};

/**
 * What the scene is drawn at: a share of the output, upscaled, and the output sharpened after while it is upscaled.
 */
export interface IRendererUpscalingSettings {
  scale: ERendererRenderScale;
  /** RCAS's sharpness, from none to its most: zero leaves the upscaled frame as resolved. */
  sharpening: number;
}

/** Drawn at the output's size; sharpened halfway once upscaled. */
export const DEFAULT_RENDERER_UPSCALING_SETTINGS: IRendererUpscalingSettings = {
  scale: ERendererRenderScale.NATIVE,
  sharpening: 0.5,
};

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
 * The grass (`CDetailManager`): planted on the GPU around the camera as the engine plants it, and drawn into the
 * G-buffer. The engine's are 49 metres round at a density of 0.6 (`r__detail_radius`, `r__detail_density`).
 */
export interface IRendererGrassSettings {
  isEnabled: boolean;
  /** How close together a slot's candidates stand, from 0.1 (sparse) to 0.99 (dense): `r__detail_density`. */
  density: number;
  /** Metres around the camera grass is planted to, from 49: `r__detail_radius`. */
  radius: number;
  /** What every planted tuft is scaled by, from 1: `r__detail_height`. */
  height: number;
}

/** The engine's own grass. */
export const DEFAULT_RENDERER_GRASS_SETTINGS: IRendererGrassSettings = {
  density: 0.6,
  height: 1,
  isEnabled: true,
  radius: 49,
};

/**
 * How a light's shadow is filtered.
 */
export enum ERendererLightShadowFilter {
  /** `shadow_hw`: four comparisons a texel apart, at `r2_ls_depth_bias` -0.0003, as vanilla draws them. */
  ENGINE = "engine",
  /** Anomaly's `shadow_pcss`: a blocker search, then a penumbra of twelve comparisons, at its -0.001 bias. */
  ANOMALY = "anomaly",
}

/**
 * The local lights a scene was given: binned into a grid over the view, and accumulated after the sun in one pass.
 */
export interface IRendererLightsSettings {
  isEnabled: boolean;
  /** Whether the level file's own lights are drawn too, which the engine does only with `r2_allow_r1_lights`. */
  isLevelLights: boolean;
  /** Whether a light the engine shadows casts its shadows, through faces drawn once into an atlas and kept. */
  isShadowed: boolean;
  shadowFilter: ERendererLightShadowFilter;
}

/** The engine's own: every spawned light, and none of the level file's. */
export const DEFAULT_RENDERER_LIGHTS_SETTINGS: IRendererLightsSettings = {
  isEnabled: true,
  isLevelLights: false,
  isShadowed: true,
  shadowFilter: ERendererLightShadowFilter.ENGINE,
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
 * How many directions and steps a pixel's horizons are searched in: XeGTAO's own quality presets.
 */
export enum ERendererAmbientOcclusionQuality {
  /** One direction, two steps each way. */
  LOW = "low",
  /** Two directions, two steps. */
  MEDIUM = "medium",
  /** Three directions, three steps, `Base`'s choice. */
  HIGH = "high",
  /** Six directions, three steps. */
  ULTRA = "ultra",
}

/**
 * Ambient occlusion from the depth of the frame, GTAO as XeGTAO computes it at half resolution: it darkens the
 * hemisphere and ambient light over the baked hemisphere occlusion, as the engine's SSAO does.
 */
export interface IRendererAmbientOcclusionSettings {
  isEnabled: boolean;
  /** Metres around a point that what stands there occludes it from. */
  radius: number;
  /** How dark the occlusion goes: one XeGTAO's own curve, zero none, two its square. */
  strength: number;
  quality: ERendererAmbientOcclusionQuality;
}

/** XeGTAO's defaults, at a metre. */
export const DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS: IRendererAmbientOcclusionSettings = {
  isEnabled: true,
  quality: ERendererAmbientOcclusionQuality.HIGH,
  radius: 1,
  strength: 1,
};

/**
 * What the renderer's features are set to: the same for every consumer, chosen as a preset and whatever was changed
 * on top of it. A feature that is off costs nothing: its passes leave the frame and its targets are freed.
 */
export interface IRendererFeatureSettings {
  ambientOcclusion: IRendererAmbientOcclusionSettings;
  antialiasing: ERendererAntialiasing;
  grass: IRendererGrassSettings;
  /** Whether every pass is timed on the GPU for the report. */
  isGpuTimed: boolean;
  lights: IRendererLightsSettings;
  lod: IRendererLodSettings;
  shadows: IRendererShadowSettings;
  upscaling: IRendererUpscalingSettings;
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
    ambientOcclusion: DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS,
    antialiasing: ERendererAntialiasing.SMAA,
    grass: DEFAULT_RENDERER_GRASS_SETTINGS,
    isGpuTimed: true,
    lights: DEFAULT_RENDERER_LIGHTS_SETTINGS,
    lod: DEFAULT_RENDERER_LOD_SETTINGS,
    shadows: DEFAULT_RENDERER_SHADOW_SETTINGS,
    upscaling: DEFAULT_RENDERER_UPSCALING_SETTINGS,
  },
  [ERendererPreset.EDITING]: {
    ambientOcclusion: { ...DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS, isEnabled: false },
    antialiasing: ERendererAntialiasing.NONE,
    grass: { ...DEFAULT_RENDERER_GRASS_SETTINGS, isEnabled: false },
    isGpuTimed: true,
    // Unshadowed, the lights cost a pass over the screen: an editor keeps seeing what lights a room.
    lights: { ...DEFAULT_RENDERER_LIGHTS_SETTINGS, isShadowed: false },
    lod: DEFAULT_RENDERER_LOD_SETTINGS,
    shadows: { ...DEFAULT_RENDERER_SHADOW_SETTINGS, isEnabled: false },
    upscaling: DEFAULT_RENDERER_UPSCALING_SETTINGS,
  },
};

/** What was changed on top of a preset, feature by feature. */
export interface IRendererFeatureOverrides {
  ambientOcclusion?: Partial<IRendererAmbientOcclusionSettings>;
  antialiasing?: ERendererAntialiasing;
  grass?: Partial<IRendererGrassSettings>;
  isGpuTimed?: boolean;
  lights?: Partial<IRendererLightsSettings>;
  lod?: Partial<IRendererLodSettings>;
  shadows?: Partial<IRendererShadowSettings>;
  upscaling?: Partial<IRendererUpscalingSettings>;
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

  const ambientOcclusion: Partial<IRendererAmbientOcclusionSettings> = toAmbientOcclusionOverrides(
    source.ambientOcclusion
  );

  if (Object.keys(ambientOcclusion).length) {
    choice.overrides.ambientOcclusion = ambientOcclusion;
  }

  const grass: Partial<IRendererGrassSettings> = toGrassOverrides(source.grass);

  if (Object.keys(grass).length) {
    choice.overrides.grass = grass;
  }

  const lights: Partial<IRendererLightsSettings> = toLightsOverrides(source.lights);

  if (Object.keys(lights).length) {
    choice.overrides.lights = lights;
  }

  const shadows: Partial<IRendererShadowSettings> = toShadowOverrides(source.shadows);

  if (Object.keys(shadows).length) {
    choice.overrides.shadows = shadows;
  }

  const upscaling: Partial<IRendererUpscalingSettings> = toUpscalingOverrides(source.upscaling);

  if (Object.keys(upscaling).length) {
    choice.overrides.upscaling = upscaling;
  }

  return choice;
}

/**
 * @param choice - A preset and what was changed on top of it.
 * @returns Every feature as the choice sets it.
 */
export function resolveRendererFeatures(choice: IRendererFeatureChoice): IRendererFeatureSettings {
  const preset: IRendererFeatureSettings = RENDERER_PRESETS[choice.preset];
  const { ambientOcclusion, antialiasing, grass, isGpuTimed, lights, lod, shadows, upscaling } = choice.overrides;

  return {
    ambientOcclusion: { ...preset.ambientOcclusion, ...ambientOcclusion },
    antialiasing: antialiasing ?? preset.antialiasing,
    grass: { ...preset.grass, ...grass },
    isGpuTimed: isGpuTimed ?? preset.isGpuTimed,
    lights: { ...preset.lights, ...lights },
    lod: { ...preset.lod, ...lod },
    shadows: { ...preset.shadows, ...shadows },
    upscaling: { ...preset.upscaling, ...upscaling },
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
    (Object.keys(preset.grass) as Array<keyof IRendererGrassSettings>).some(
      (key) => resolved.grass[key] !== preset.grass[key]
    ) ||
    (Object.keys(preset.lights) as Array<keyof IRendererLightsSettings>).some(
      (key) => resolved.lights[key] !== preset.lights[key]
    ) ||
    (Object.keys(preset.upscaling) as Array<keyof IRendererUpscalingSettings>).some(
      (key) => resolved.upscaling[key] !== preset.upscaling[key]
    ) ||
    (Object.keys(preset.ambientOcclusion) as Array<keyof IRendererAmbientOcclusionSettings>).some(
      (key) => resolved.ambientOcclusion[key] !== preset.ambientOcclusion[key]
    ) ||
    (Object.keys(preset.shadows) as Array<keyof IRendererShadowSettings>).some((key) =>
      key === "cascades"
        ? resolved.shadows.cascades.join() !== preset.shadows.cascades.join()
        : resolved.shadows[key] !== preset.shadows[key]
    )
  );
}

/**
 * @param stored - What was stored for the grass overrides.
 * @returns The ones the grass takes: a flag, and finite numbers not below zero.
 */
function toGrassOverrides(stored: unknown): Partial<IRendererGrassSettings> {
  const source: Record<string, unknown> = stored && typeof stored === "object" ? (stored as never) : {};
  const overrides: Partial<IRendererGrassSettings> = {};

  if (typeof source.isEnabled === "boolean") {
    overrides.isEnabled = source.isEnabled;
  }

  for (const key of ["density", "height", "radius"] as const) {
    const value: unknown = source[key];

    if (typeof value === "number" && Number.isFinite(value) && value > 0) {
      overrides[key] = value;
    }
  }

  return overrides;
}

/**
 * @param stored - What was stored for the lights overrides.
 * @returns The ones the lights take: their flags.
 */
function toLightsOverrides(stored: unknown): Partial<IRendererLightsSettings> {
  const source: Record<string, unknown> = stored && typeof stored === "object" ? (stored as never) : {};
  const overrides: Partial<IRendererLightsSettings> = {};

  for (const key of ["isEnabled", "isLevelLights", "isShadowed"] as const) {
    if (typeof source[key] === "boolean") {
      overrides[key] = source[key];
    }
  }

  const shadowFilter: ERendererLightShadowFilter | undefined = Object.values(ERendererLightShadowFilter).find(
    (it) => it === source.shadowFilter
  );

  if (shadowFilter) {
    overrides.shadowFilter = shadowFilter;
  }

  return overrides;
}

/**
 * @param stored - What was stored for the ambient occlusion overrides.
 * @returns The ones the ambient occlusion takes: a flag, finite numbers not below zero, and a quality it has.
 */
function toAmbientOcclusionOverrides(stored: unknown): Partial<IRendererAmbientOcclusionSettings> {
  const source: Record<string, unknown> = stored && typeof stored === "object" ? (stored as never) : {};
  const overrides: Partial<IRendererAmbientOcclusionSettings> = {};
  const quality: ERendererAmbientOcclusionQuality | undefined = Object.values(ERendererAmbientOcclusionQuality).find(
    (it) => it === source.quality
  );

  if (typeof source.isEnabled === "boolean") {
    overrides.isEnabled = source.isEnabled;
  }

  for (const key of ["radius", "strength"] as const) {
    const value: unknown = source[key];

    if (typeof value === "number" && Number.isFinite(value) && value >= 0) {
      overrides[key] = value;
    }
  }

  if (quality) {
    overrides.quality = quality;
  }

  return overrides;
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

/**
 * @param stored - What was stored for the upscaling overrides.
 * @returns The ones the upscaling takes: a render scale it has, and a sharpening from zero to one.
 */
function toUpscalingOverrides(stored: unknown): Partial<IRendererUpscalingSettings> {
  const source: Record<string, unknown> = stored && typeof stored === "object" ? (stored as never) : {};
  const overrides: Partial<IRendererUpscalingSettings> = {};
  const scale: ERendererRenderScale | undefined = Object.values(ERendererRenderScale).find((it) => it === source.scale);

  if (scale) {
    overrides.scale = scale;
  }

  if (typeof source.sharpening === "number" && source.sharpening >= 0 && source.sharpening <= 1) {
    overrides.sharpening = source.sharpening;
  }

  return overrides;
}

/**
 * @param features - What the features are set to.
 * @returns The ratio of the output's side to the scene's as drawn.
 */
export function toRendererUpscale(features: IRendererFeatureSettings): number {
  return RENDERER_RENDER_SCALE_RATIOS[features.upscaling.scale];
}
