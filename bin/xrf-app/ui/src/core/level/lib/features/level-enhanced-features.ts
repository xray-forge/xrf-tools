import {
  explainRenderDebandingMode,
  RENDER_DEBANDING_SETTINGS,
} from "@/core/render/lib/features/image/render-debanding-choices";
import {
  explainRenderVolumetricMode,
  RENDER_VOLUMETRIC_SETTINGS,
} from "@/core/render/lib/features/lighting/render-volumetric-choices";
import { TRenderSettings } from "@/core/render/lib/features/render-setting";
import {
  explainRenderIndirectLightMode,
  RENDER_INDIRECT_LIGHT_SETTINGS,
} from "@/core/render/lib/features/tracing/render-indirect-light-choices";
import { explainRenderGlassMode, RENDER_GLASS_SETTINGS } from "@/core/render/lib/features/world/render-glass-choices";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

/** The view overrides drawn either as the engine draws them or enhanced, whose every value tunes the enhanced. */
export type TLevelEnhancedFeatureKey = "debanding" | "glass" | "indirectLight" | "volumetric";

/** How one of them is drawn: `engine` or `enhanced`. */
export type TLevelEnhancedFeatureMode<K extends TLevelEnhancedFeatureKey> = IRenderFeatureSettings[K]["mode"];

/** How a popover offers one of them: a toggle for the enhanced mode over the values tuning it. */
export interface ILevelEnhancedFeature<K extends TLevelEnhancedFeatureKey> {
  /** Names the toggle that draws it enhanced. */
  label: string;
  /** What it is called where its reset is named. */
  subject: string;
  /** What a mode draws, said under the toggle. */
  explain: (mode: TLevelEnhancedFeatureMode<K>) => string;
  /** How each of its settings is set. */
  settings: TRenderSettings<IRenderFeatureSettings[K]>;
  /** The values tuning the enhanced, as the popover lists them. */
  names: ReadonlyArray<Exclude<keyof IRenderFeatureSettings[K], "mode">>;
}

/** Each feature drawn as the engine does or enhanced, as its popover offers it. */
export const LEVEL_ENHANCED_FEATURES: { readonly [K in TLevelEnhancedFeatureKey]: ILevelEnhancedFeature<K> } = {
  debanding: {
    explain: explainRenderDebandingMode,
    label: "Debanding",
    names: ["passes", "radius"],
    settings: RENDER_DEBANDING_SETTINGS,
    subject: "debanding",
  },
  glass: {
    explain: explainRenderGlassMode,
    label: "Enhanced glass",
    names: ["refraction"],
    settings: RENDER_GLASS_SETTINGS,
    subject: "glass",
  },
  indirectLight: {
    explain: explainRenderIndirectLightMode,
    label: "Indirect light",
    names: ["intensity", "radius", "steps"],
    settings: RENDER_INDIRECT_LIGHT_SETTINGS,
    subject: "indirect light",
  },
  volumetric: {
    explain: explainRenderVolumetricMode,
    label: "Enhanced volumetric light",
    names: ["intensity", "anisotropy", "distance"],
    settings: RENDER_VOLUMETRIC_SETTINGS,
    subject: "volumetric light",
  },
};
