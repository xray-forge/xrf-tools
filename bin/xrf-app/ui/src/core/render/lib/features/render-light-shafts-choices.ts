import { formatNumber } from "@xrf/format";
import { ERenderLightShafts, RenderLightShafts } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { IRenderLimits, toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";

const RENDER_LIGHT_SHAFTS_DESCRIPTIONS: Readonly<Record<RenderLightShafts, string>> = {
  [ERenderLightShafts.ENGINE]:
    "The lamps the game gives shafts: those the level asks for, or every lamp under Anomaly's engine.",
  [ERenderLightShafts.SHADOWED]:
    "Every shadowed lamp, as Anomaly forces them on: shafts on a level whose lamps ask for none.",
};

/**
 * @param shafts - Which lamps draw shafts.
 * @returns What it does, in a sentence.
 */
export function explainRenderLightShafts(shafts: RenderLightShafts): string {
  return RENDER_LIGHT_SHAFTS_DESCRIPTIONS[shafts];
}

/** Which lamps draw shafts, in the order offered. */
export const RENDER_LIGHT_SHAFTS_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderLightShafts>> = [
  { label: "Engine", value: ERenderLightShafts.ENGINE },
  { label: "Every lamp", value: ERenderLightShafts.SHADOWED },
];

/** How far the lamps' shafts are scaled, from none to four times the game's. */
export const RENDER_SHAFT_STRENGTH_LIMITS: IRenderLimits = toRenderLimits(
  RENDER_FEATURE_SCHEMA.lights.shaftStrength,
  0.05
);

/** @returns A shaft strength as a share of the game's. */
export function formatShaftStrength(strength: number): string {
  return `${formatNumber(strength * 100, 0)}%`;
}
