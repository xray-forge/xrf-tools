import { ERendererAmbientOcclusionQuality } from "@/core/render/lib/contract/renderer-ambient-occlusion-quality";
import { ERendererAntialiasing } from "@/core/render/lib/contract/renderer-antialiasing";
import { IRendererChoiceField } from "@/core/render/lib/contract/renderer-choice-field";
import { IRendererFeatureSettings } from "@/core/render/lib/contract/renderer-feature-settings";
import { IRendererFlagField } from "@/core/render/lib/contract/renderer-flag-field";
import { ERendererLightShadowFilter } from "@/core/render/lib/contract/renderer-light-shadow-filter";
import { IRendererNumberField } from "@/core/render/lib/contract/renderer-number-field";
import { ERendererRenderScale } from "@/core/render/lib/contract/renderer-render-scale";
import { ERendererSettingKind } from "@/core/render/lib/contract/renderer-setting-field";
import { TRendererSettingSchema } from "@/core/render/lib/contract/renderer-setting-schema";
import { RENDERER_MAX_SHADOW_CASCADES } from "@/core/render/lib/contract/renderer-shadow-settings";

const FLAG: IRendererFlagField = { kind: ERendererSettingKind.FLAG };

function toNumber(min: number, max: number, isInteger: boolean = false): IRendererNumberField {
  return { isInteger, kind: ERendererSettingKind.NUMBER, max, min };
}

function toChoice<T extends string>(values: Record<string, T>): IRendererChoiceField<T> {
  return { kind: ERendererSettingKind.CHOICE, values: Object.values(values) };
}

/**
 * Every feature setting, what it takes, and between which bounds: the engine console's own where the engine has the
 * setting (`xrRender_console.cpp`), the renderer's where it does not.
 */
export const RENDERER_FEATURE_SCHEMA: TRendererSettingSchema<IRendererFeatureSettings> = {
  ambientOcclusion: {
    isEnabled: FLAG,
    quality: toChoice(ERendererAmbientOcclusionQuality),
    radius: toNumber(0.1, 8),
    strength: toNumber(0, 2),
  },
  antialiasing: toChoice(ERendererAntialiasing),
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
    height: toNumber(0.5, 2),
    isEnabled: FLAG,
    radius: toNumber(49, 300, true),
  },
  isOcclusionCulled: FLAG,
  lights: {
    isEnabled: FLAG,
    isLevelLights: FLAG,
    isShadowed: FLAG,
    shadowFilter: toChoice(ERendererLightShadowFilter),
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
    cascades: { kind: ERendererSettingKind.WIDTHS, max: 2000, min: 1, most: RENDERER_MAX_SHADOW_CASCADES },
    filter: toNumber(0, 3, true),
    isEnabled: FLAG,
    isStaggered: FLAG,
    reach: toNumber(0, 2000),
    resolution: toNumber(256, 8192, true),
  },
  upscaling: {
    scale: toChoice(ERendererRenderScale),
    sharpening: toNumber(0, 1),
  },
  water: {
    distortion: toNumber(0, 0.2),
    isDistorted: FLAG,
    isEnabled: FLAG,
    isSoft: FLAG,
    reflection: toNumber(0, 4),
    ripple: toNumber(0, 4),
    waveHeight: toNumber(0, 0.2),
    waveSpeed: toNumber(0, 100),
  },
};
