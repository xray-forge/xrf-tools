import {
  ERenderAmbientOcclusionMethod,
  ERenderAmbientOcclusionQuality,
  ERenderAntialiasing,
  ERenderContactShadowMode,
  ERenderFoliageMode,
  ERenderIndirectLightMode,
  ERenderLightShadowFilter,
  ERenderReflectionMode,
  ERenderReflectionQuality,
  ERenderScale,
  ERenderWaterMode,
} from "@/core/ipc/types/xrf-renderer";
import {
  RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION,
  RENDER_MAX_CONTACT_SHADOW_LIGHTS,
  RENDER_MAX_SHADOW_CASCADES,
} from "@/core/render/lib/settings/render-feature-defaults";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import {
  ERenderSettingKind,
  IRenderChoiceField,
  IRenderFlagField,
  IRenderNumberField,
} from "@/core/render/lib/settings/render-setting-field";
import { TRenderSettingSchema } from "@/core/render/lib/settings/render-setting-schema";

const FLAG: IRenderFlagField = { kind: ERenderSettingKind.FLAG };

function toNumber(min: number, max: number, isInteger: boolean = false): IRenderNumberField {
  return { isInteger, kind: ERenderSettingKind.NUMBER, max, min };
}

function toChoice<T extends string>(values: Record<string, T>): IRenderChoiceField<T> {
  return { kind: ERenderSettingKind.CHOICE, values: Object.values(values) };
}

/**
 * Every feature setting, what it takes, and between which bounds: the engine console's own where the engine has the
 * setting (`xrRender_console.cpp`), the renderer's where it does not.
 */
export const RENDER_FEATURE_SCHEMA: TRenderSettingSchema<IRenderFeatureSettings> = {
  ambientOcclusion: {
    isEnabled: FLAG,
    method: toChoice(ERenderAmbientOcclusionMethod),
    quality: toChoice(ERenderAmbientOcclusionQuality),
    radius: toNumber(0.1, 8),
    strength: toNumber(0, 2),
    // The renderer's own: past two metres a thickness lets nothing through that a horizon would not.
    vbao: {
      accumulation: toNumber(1, RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION, true),
      bounce: toNumber(0, 1),
      thickness: toNumber(0.01, 2),
    },
  },
  antialiasing: toChoice(ERenderAntialiasing),
  // The console's own bounds.
  exposure: {
    adaptation: toNumber(0.01, 10),
    amount: toNumber(0, 1),
    isEnabled: FLAG,
    lowLuminance: toNumber(0.0001, 1),
    middleGray: toNumber(0, 2),
  },
  grass: {
    // `r__detail_density`: a spacing. The console goes to 0.1, six times the game's; past three times it at the widest
    // radius the grass's buffers outgrow what the GPU process holds and the renderer crashes.
    density: toNumber(0.2, 0.99),
    // The enhanced motion's bounds are the engine console's.
    foliage: {
      grassPush: toNumber(0, 5),
      grassSpeed: toNumber(0, 20),
      grassTurbulence: toNumber(0, 5),
      grassWave: toNumber(0, 5),
      minSpeed: toNumber(0, 1),
      mode: toChoice(ERenderFoliageMode),
      sssColor: toNumber(0, 1),
      sssIntensity: toNumber(0, 5),
      treesBend: toNumber(0, 5),
      treesSpeed: toNumber(0, 20),
      treesTrunk: toNumber(0, 5),
    },
    height: toNumber(0.5, 2),
    isEnabled: FLAG,
    radius: toNumber(49, 300, true),
  },
  // The renderer's own: up to four times what the frame shows, for the light a screen-space search cannot see; past
  // eight metres its steps lie too far apart to follow a surface.
  indirectLight: {
    intensity: toNumber(0, 4),
    mode: toChoice(ERenderIndirectLightMode),
    radius: toNumber(0.5, 8),
  },
  isOcclusionCulled: FLAG,
  // The renderer's own: a hit replaces at most all of the cube, never more, so nothing brightens; past two hundred
  // metres a ray leaves the frame long before its steps reach.
  reflections: {
    distance: toNumber(5, 200),
    intensity: toNumber(0, 1),
    mode: toChoice(ERenderReflectionMode),
    quality: toChoice(ERenderReflectionQuality),
  },
  lights: {
    isEnabled: FLAG,
    isLevelLights: FLAG,
    isShadowed: FLAG,
    shadowFilter: toChoice(ERenderLightShadowFilter),
  },
  lod: {
    geometryLod: toNumber(0.1, 2),
    isImpostors: FLAG,
    ssaA: toNumber(16, 96),
    ssaB: toNumber(32, 64),
    ssaDiscard: toNumber(1, 10),
    ssaGlodEnd: toNumber(16, 96),
    ssaGlodStart: toNumber(128, 512),
  },
  shadows: {
    bias: toNumber(0, 5),
    // Kept short of the map's middle, where every point would blend.
    blend: toNumber(0, 0.4),
    cascades: { kind: ERenderSettingKind.WIDTHS, max: 2000, min: 1, most: RENDER_MAX_SHADOW_CASCADES },
    // The renderer's own: past a few metres a ray leaves what screen space can see.
    contact: {
      intensity: toNumber(0, 1),
      length: toNumber(0, 4),
      lights: toNumber(0, RENDER_MAX_CONTACT_SHADOW_LIGHTS, true),
      mode: toChoice(ERenderContactShadowMode),
      steps: toNumber(4, 64, true),
      thickness: toNumber(0.01, 1),
    },
    filter: toNumber(0, 3, true),
    isEnabled: FLAG,
    isStaggered: FLAG,
    reach: toNumber(0, 2000),
    resolution: toNumber(256, 8192, true),
  },
  upscaling: {
    scale: toChoice(ERenderScale),
    sharpening: toNumber(0, 1),
  },
  water: {
    distortion: toNumber(0, 0.2),
    enhanced: {
      blurNoise: toNumber(0, 1),
      calmFlow: toNumber(0, 1),
      caustics: toNumber(0, 1),
      flow: toNumber(0, 2),
      parallaxHeight: toNumber(0, 0.1),
      reflectionBlur: toNumber(0, 1),
      reflectivity: toNumber(0, 1),
      refraction: toNumber(0, 2),
      ripples: toNumber(0, 1),
      softBorder: toNumber(0, 1),
      specular: toNumber(0, 10),
      turbidity: toNumber(0, 10),
      variation: toNumber(0, 1),
    },
    isDistorted: FLAG,
    isEnabled: FLAG,
    isSoft: FLAG,
    mode: toChoice(ERenderWaterMode),
    reflection: toNumber(0, 4),
    ripple: toNumber(0, 4),
    waveHeight: toNumber(0, 0.2),
    waveSpeed: toNumber(0, 100),
  },
};
