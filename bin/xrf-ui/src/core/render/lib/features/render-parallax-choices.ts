import { formatNumber, formatPercent } from "@xrf/format";

import { ERenderParallaxMode, RenderParallaxMode } from "@/core/ipc/types/xrf-renderer";
import { IRenderLimits } from "@/core/render/lib/features/render-limits";
import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow/choice-form-row-option";

/** The bounds each enhanced parallax value is offered between: narrower than the renderer takes, where relief reads. */
export const RENDER_PARALLAX_LIMITS = {
  blending: { max: 1, min: 0, step: 0.05 },
  cavity: { max: 2, min: 0, step: 0.05 },
  height: { max: 0.12, min: 0.005, step: 0.005 },
  range: { max: 60, min: 5, step: 1 },
  shadow: { max: 1, min: 0, step: 0.05 },
  steps: { max: 48, min: 4, step: 1 },
  surfaceHeight: { max: 0.06, min: 0.005, step: 0.005 },
  surfaceRange: { max: 100, min: 4, step: 1 },
} as const satisfies Record<string, IRenderLimits>;

const RENDER_PARALLAX_MODE_DESCRIPTIONS: Readonly<Record<RenderParallaxMode, string>> = {
  [ERenderParallaxMode.ENGINE]:
    "The game's own: steep parallax on surfaces whose texture asks for it, none on terrain.",
  [ERenderParallaxMode.ENHANCED]:
    "Terrain and bumped surfaces marched over their heights, shadowed towards the sun, terrain layers blended by height.",
};

/**
 * @param mode - A parallax mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderParallaxMode(mode: RenderParallaxMode): string {
  return RENDER_PARALLAX_MODE_DESCRIPTIONS[mode];
}

/** The parallax modes, in the order they are offered. */
export const RENDER_PARALLAX_MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderParallaxMode>> = [
  { label: "Engine", value: ERenderParallaxMode.ENGINE },
  { label: "Enhanced", value: ERenderParallaxMode.ENHANCED },
];

/** @returns How many steps a march takes at most. */
export function formatParallaxSteps(steps: number): string {
  return formatNumber(steps, 0);
}

/** @returns How far the relief is marched. */
export function formatParallaxRange(range: number): string {
  return `${formatNumber(range, 0)} m`;
}

/** @returns How deep the relief reaches, in texture coordinates. */
export function formatParallaxHeight(height: number): string {
  return formatNumber(height, 3);
}

/** @returns How strongly the relief shadows, blends or darkens. */
export function formatParallaxStrength(strength: number): string {
  return formatPercent(strength);
}
