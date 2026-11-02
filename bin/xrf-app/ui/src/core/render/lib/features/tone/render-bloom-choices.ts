import { formatNumber } from "@xrf/format";
import { ERenderBloomMode, RenderBloomMode } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { IRenderLimits, toRenderLimits } from "@/core/render/lib/features/render-limits";
import {
  ERenderSettingControl,
  IRenderSliderSetting,
  TRenderSettings,
} from "@/core/render/lib/features/render-setting";
import { TRenderEnhancedBloomSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/schema/render-feature-schema";

const RENDER_BLOOM_MODE_DESCRIPTIONS: Readonly<Record<RenderBloomMode, string>> = {
  [ERenderBloomMode.ENGINE]: "Draws the game's bloom, as the look's bloom settings set it.",
  [ERenderBloomMode.ENHANCED]:
    "Blurs the frame's bright parts and self-lit surfaces wide and screens them over it, in place of the game's bloom.",
};

/**
 * @param mode - A bloom mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderBloomMode(mode: RenderBloomMode): string {
  return RENDER_BLOOM_MODE_DESCRIPTIONS[mode];
}

/** The bloom modes, in the order they are offered. */
export const RENDER_BLOOM_MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderBloomMode>> = [
  { label: "Engine", value: ERenderBloomMode.ENGINE },
  { label: "Enhanced", value: ERenderBloomMode.ENHANCED },
];

/** @returns One of the enhanced bloom's strengths. */
export function formatBloomStrength(strength: number): string {
  return formatNumber(strength, 1);
}

/**
 * @param label - What it is called.
 * @param description - What it does.
 * @param limits - Between what it is offered.
 * @returns One of the enhanced bloom's strengths, which tune nothing of the game's bloom.
 */
function toEnhancedBloomSlider(
  label: string,
  description: string,
  limits: IRenderLimits
): IRenderSliderSetting<TRenderEnhancedBloomSettings> {
  return {
    control: ERenderSettingControl.SLIDER,
    description,
    format: formatBloomStrength,
    isInert: (bloom: TRenderEnhancedBloomSettings) => bloom.mode === ERenderBloomMode.ENGINE,
    label,
    limits,
  };
}

/** How each enhanced bloom setting is set, in the settings and the Look popover alike. */
export const RENDER_ENHANCED_BLOOM_SETTINGS: TRenderSettings<TRenderEnhancedBloomSettings> = {
  blur: toEnhancedBloomSlider(
    "Blur",
    "How wide the glow spreads.",
    toRenderLimits(RENDER_FEATURE_SCHEMA.enhancedBloom.blur, 0.1)
  ),
  exposure: toEnhancedBloomSlider(
    "Brightness",
    "How bright the glow is.",
    toRenderLimits(RENDER_FEATURE_SCHEMA.enhancedBloom.exposure, 0.5)
  ),
  mode: {
    control: ERenderSettingControl.CHOICE,
    explain: explainRenderBloomMode,
    label: "Mode",
    options: RENDER_BLOOM_MODE_OPTIONS,
  },
  sky: toEnhancedBloomSlider(
    "Sky",
    "How much the sky glows against the rest of the frame.",
    toRenderLimits(RENDER_FEATURE_SCHEMA.enhancedBloom.sky, 0.1)
  ),
  threshold: toEnhancedBloomSlider(
    "Threshold",
    "How bright a part must be to glow: the higher, the less glows.",
    toRenderLimits(RENDER_FEATURE_SCHEMA.enhancedBloom.threshold, 0.1)
  ),
  vibrance: toEnhancedBloomSlider(
    "Vibrance",
    "How saturated the glow is: 1 as the frame's colour, 0 grey.",
    toRenderLimits(RENDER_FEATURE_SCHEMA.enhancedBloom.vibrance, 0.1)
  ),
};
