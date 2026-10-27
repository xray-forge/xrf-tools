import { formatNumber } from "@xrf/format";
import { ERenderBloomMode, RenderBloomMode } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/ChoiceFormRow/choice-form-row-option";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";

/** The bounds each enhanced bloom value is offered between. */
export const RENDER_ENHANCED_BLOOM_LIMITS = {
  blur: toRenderLimits(RENDER_FEATURE_SCHEMA.enhancedBloom.blur, 0.1),
  exposure: toRenderLimits(RENDER_FEATURE_SCHEMA.enhancedBloom.exposure, 0.5),
  sky: toRenderLimits(RENDER_FEATURE_SCHEMA.enhancedBloom.sky, 0.1),
  threshold: toRenderLimits(RENDER_FEATURE_SCHEMA.enhancedBloom.threshold, 0.1),
  vibrance: toRenderLimits(RENDER_FEATURE_SCHEMA.enhancedBloom.vibrance, 0.1),
} as const;

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
