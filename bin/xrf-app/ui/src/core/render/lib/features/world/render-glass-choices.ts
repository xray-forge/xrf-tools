import { formatNumber } from "@xrf/format";
import { ERenderGlassMode, RenderGlassMode } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { ERenderSettingControl, TRenderSettings } from "@/core/render/lib/features/render-setting";
import { TRenderGlassSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/schema/render-feature-schema";

const RENDER_GLASS_MODE_DESCRIPTIONS: Readonly<Record<RenderGlassMode, string>> = {
  [ERenderGlassMode.ENGINE]: "Blends glass over what is behind it, mixed toward the sky's cube, as the game does.",
  [ERenderGlassMode.ENHANCED]:
    "Refracts what is behind the glass, blurred where it is dirty, and reflects the sky and the sun's highlight.",
};

/**
 * @param mode - A glass mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderGlassMode(mode: RenderGlassMode): string {
  return RENDER_GLASS_MODE_DESCRIPTIONS[mode];
}

/** The glass modes, in the order they are offered. */
export const RENDER_GLASS_MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderGlassMode>> = [
  { label: "Engine", value: ERenderGlassMode.ENGINE },
  { label: "Enhanced", value: ERenderGlassMode.ENHANCED },
];

/** @returns One of the enhanced glass's strengths. */
export function formatGlassStrength(strength: number): string {
  return formatNumber(strength, 2);
}

/** How each glass setting is set, in the settings and the Reflections popover alike. */
export const RENDER_GLASS_SETTINGS: TRenderSettings<TRenderGlassSettings> = {
  mode: {
    control: ERenderSettingControl.CHOICE,
    explain: explainRenderGlassMode,
    label: "Mode",
    options: RENDER_GLASS_MODE_OPTIONS,
  },
  refraction: {
    control: ERenderSettingControl.SLIDER,
    description: "How far the glass bends what is behind it: 1 as designed, 0 flat.",
    format: formatGlassStrength,
    isInert: (glass: TRenderGlassSettings) => glass.mode === ERenderGlassMode.ENGINE,
    label: "Refraction",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.glass.refraction, 0.05),
  },
};
