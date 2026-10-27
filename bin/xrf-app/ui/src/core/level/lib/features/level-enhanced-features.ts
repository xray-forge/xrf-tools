import { formatCount } from "@xrf/format";

import { formatOcclusionRadius } from "@/core/render/lib/features/render-ambient-occlusion-choices";
import {
  explainRenderDebandingMode,
  formatDebandingPasses,
  formatDebandingRadius,
  RENDER_DEBANDING_LIMITS,
} from "@/core/render/lib/features/render-debanding-choices";
import {
  explainRenderGlassMode,
  formatGlassStrength,
  RENDER_GLASS_LIMITS,
} from "@/core/render/lib/features/render-glass-choices";
import {
  explainRenderIndirectLightMode,
  formatIndirectLightIntensity,
  RENDER_INDIRECT_LIGHT_LIMITS,
} from "@/core/render/lib/features/render-indirect-light-choices";
import { IRenderLimits } from "@/core/render/lib/features/render-limits";
import {
  explainRenderVolumetricMode,
  formatVolumetricDistance,
  formatVolumetricStrength,
  RENDER_VOLUMETRIC_LIMITS,
} from "@/core/render/lib/features/render-volumetric-choices";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";

/** The view overrides drawn either as the engine draws them or enhanced, whose every value tunes the enhanced. */
export type TLevelEnhancedFeatureKey = "debanding" | "glass" | "indirectLight" | "volumetric";

/** How one of them is drawn: `engine` or `enhanced`. */
export type TLevelEnhancedFeatureMode<K extends TLevelEnhancedFeatureKey> = IRenderFeatureSettings[K]["mode"];

/** One of an enhanced feature's values, as a slider sets it. */
export interface ILevelEnhancedFeatureSlider<K extends TLevelEnhancedFeatureKey> {
  key: Exclude<keyof IRenderFeatureSettings[K], "mode">;
  label: string;
  limits: IRenderLimits;
  format: (value: number) => string;
}

/** How a popover offers one of them. */
export interface ILevelEnhancedFeature<K extends TLevelEnhancedFeatureKey> {
  /** Names the toggle that draws it enhanced. */
  label: string;
  /** What it is called where its reset is named. */
  subject: string;
  /** What a mode draws, said under the toggle. */
  explain: (mode: TLevelEnhancedFeatureMode<K>) => string;
  sliders: ReadonlyArray<ILevelEnhancedFeatureSlider<K>>;
}

/** Each feature drawn as the engine does or enhanced, as its popover offers it. */
export const LEVEL_ENHANCED_FEATURES: { readonly [K in TLevelEnhancedFeatureKey]: ILevelEnhancedFeature<K> } = {
  debanding: {
    explain: explainRenderDebandingMode,
    label: "Debanding",
    sliders: [
      {
        format: formatDebandingPasses,
        key: "passes",
        label: "Debanding passes",
        limits: RENDER_DEBANDING_LIMITS.passes,
      },
      {
        format: formatDebandingRadius,
        key: "radius",
        label: "Debanding radius",
        limits: RENDER_DEBANDING_LIMITS.radius,
      },
    ],
    subject: "debanding",
  },
  glass: {
    explain: explainRenderGlassMode,
    label: "Enhanced glass",
    sliders: [
      { format: formatGlassStrength, key: "refraction", label: "Refraction", limits: RENDER_GLASS_LIMITS.refraction },
    ],
    subject: "glass",
  },
  indirectLight: {
    explain: explainRenderIndirectLightMode,
    label: "Indirect light",
    sliders: [
      {
        format: formatIndirectLightIntensity,
        key: "intensity",
        label: "Intensity",
        limits: RENDER_INDIRECT_LIGHT_LIMITS.intensity,
      },
      {
        format: formatOcclusionRadius,
        key: "radius",
        label: "Bounce radius",
        limits: RENDER_INDIRECT_LIGHT_LIMITS.radius,
      },
      { format: formatCount, key: "steps", label: "Bounce steps", limits: RENDER_INDIRECT_LIGHT_LIMITS.steps },
    ],
    subject: "indirect light",
  },
  volumetric: {
    explain: explainRenderVolumetricMode,
    label: "Enhanced volumetric light",
    sliders: [
      {
        format: formatVolumetricStrength,
        key: "intensity",
        label: "Intensity",
        limits: RENDER_VOLUMETRIC_LIMITS.intensity,
      },
      {
        format: formatVolumetricStrength,
        key: "anisotropy",
        label: "Anisotropy",
        limits: RENDER_VOLUMETRIC_LIMITS.anisotropy,
      },
      {
        format: formatVolumetricDistance,
        key: "distance",
        label: "Distance",
        limits: RENDER_VOLUMETRIC_LIMITS.distance,
      },
    ],
    subject: "volumetric light",
  },
};
