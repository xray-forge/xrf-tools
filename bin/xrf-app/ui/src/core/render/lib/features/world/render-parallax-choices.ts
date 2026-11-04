import { formatNumber, formatPercent } from "@xrf/format";
import { ERenderParallaxMode, RenderParallaxMode } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { IRenderLimits } from "@/core/render/lib/features/render-limits";
import { ERenderSettingControl, TRenderSettings } from "@/core/render/lib/features/render-setting";
import { TRenderParallaxSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

/** The bounds each enhanced parallax value is offered between: narrower than the renderer takes, where relief reads. */
const RENDER_PARALLAX_LIMITS = {
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
    "Draws steep parallax on surfaces whose texture asks for it and none on terrain, as the game does.",
  [ERenderParallaxMode.ENHANCED]:
    "Marches terrain and bumped surfaces over their heights, shadows them towards the sun, blends terrain by height.",
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

/**
 * @param parallax - The parallax a view resolves to.
 * @returns Whether it displaces anything: the enhanced march, or the engine's steep parallax while that is on.
 */
export function isRenderParallaxDrawn(parallax: TRenderParallaxSettings): boolean {
  return parallax.isSteep || parallax.mode === ERenderParallaxMode.ENHANCED;
}

/** Whether the parallax is the engine's steep parallax alone, which none of the enhanced march's values tune. */
function isEngineParallax(parallax: TRenderParallaxSettings): boolean {
  return parallax.mode === ERenderParallaxMode.ENGINE;
}

/** How each parallax setting is set, in the settings and the Relief popover alike. */
export const RENDER_PARALLAX_SETTINGS: TRenderSettings<TRenderParallaxSettings> = {
  blending: {
    control: ERenderSettingControl.SLIDER,
    description: "How far the terrain's layers blend by height, the one standing highest showing through.",
    format: formatParallaxStrength,
    isInert: isEngineParallax,
    label: "Height blending",
    limits: RENDER_PARALLAX_LIMITS.blending,
  },
  cavity: {
    control: ERenderSettingControl.SLIDER,
    description: "How much the low ground of the relief darkens.",
    format: formatParallaxStrength,
    isInert: isEngineParallax,
    label: "Cavities",
    limits: RENDER_PARALLAX_LIMITS.cavity,
  },
  height: {
    control: ERenderSettingControl.SLIDER,
    description: "How deep the relief reaches, in the detail's texture coordinates.",
    format: formatParallaxHeight,
    isInert: isEngineParallax,
    label: "Height",
    limits: RENDER_PARALLAX_LIMITS.height,
  },
  isRefined: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Settles each march where it met the ground by halving its last step three times.",
    isInert: isEngineParallax,
    label: "Refined",
  },
  isSteep: {
    control: ERenderSettingControl.CHECKBOX,
    description: "The engine's steep parallax within 12 m on level surfaces whose texture asks for it.",
    isEntryDescribed: true,
    label: "Steep parallax",
  },
  mode: {
    control: ERenderSettingControl.CHOICE,
    explain: explainRenderParallaxMode,
    isEntryDescribed: true,
    label: "Mode",
    options: RENDER_PARALLAX_MODE_OPTIONS,
  },
  range: {
    control: ERenderSettingControl.SLIDER,
    description: "Metres within which the terrain is marched, faded out over the last fifth.",
    format: formatParallaxRange,
    isInert: isEngineParallax,
    label: "Range",
    limits: RENDER_PARALLAX_LIMITS.range,
  },
  shadow: {
    control: ERenderSettingControl.SLIDER,
    description: "How much of the sun the relief keeps from its own hollows.",
    format: formatParallaxStrength,
    isInert: isEngineParallax,
    label: "Self-shadowing",
    limits: RENDER_PARALLAX_LIMITS.shadow,
  },
  steps: {
    control: ERenderSettingControl.SLIDER,
    description: "Steps a terrain march takes at most, looking along the ground; three looking straight down.",
    format: formatParallaxSteps,
    isInert: isEngineParallax,
    label: "Steps",
    limits: RENDER_PARALLAX_LIMITS.steps,
  },
  surfaceHeight: {
    control: ERenderSettingControl.SLIDER,
    description: "How deep the relief reaches on level surfaces whose texture asks for parallax.",
    format: formatParallaxHeight,
    isInert: isEngineParallax,
    label: "Surface height",
    limits: RENDER_PARALLAX_LIMITS.surfaceHeight,
  },
  surfaceRange: {
    control: ERenderSettingControl.SLIDER,
    description: "Metres within which those surfaces are marched, faded out over the last fifth.",
    format: formatParallaxRange,
    isInert: isEngineParallax,
    label: "Surface range",
    limits: RENDER_PARALLAX_LIMITS.surfaceRange,
  },
};
