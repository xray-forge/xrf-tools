import { formatNumber } from "@xrf/format";

import { ERenderGlassMode, RenderGlassMode } from "@/core/ipc/types/xrf-renderer";
import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";
import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow/choice-form-row-option";

/** The bounds the enhanced glass's refraction is offered between. */
export const RENDER_GLASS_LIMITS = {
  refraction: toRenderLimits(RENDER_FEATURE_SCHEMA.glass.refraction, 0.05),
} as const;

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
