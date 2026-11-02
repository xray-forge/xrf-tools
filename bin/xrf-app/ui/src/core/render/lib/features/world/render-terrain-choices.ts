import { formatNumber } from "@xrf/format";
import { ERenderTerrainTiling, RenderTerrainTiling } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { IRenderLimits } from "@/core/render/lib/features/render-limits";
import { ERenderSettingControl, TRenderSettings } from "@/core/render/lib/features/render-setting";
import { TRenderTerrainSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

/**
 * The bounds each varied tiling value is offered between: narrower than the renderer takes, where the tiling reads.
 * Past four its hexagons shrink to a blur of copies; past sixteen their seams show as hard edges.
 */
const RENDER_TERRAIN_LIMITS = {
  rotation: { max: 1, min: 0, step: 0.05 },
  scale: { max: 4, min: 0.5, step: 0.25 },
  sharpness: { max: 16, min: 1, step: 0.5 },
} as const satisfies Record<string, IRenderLimits>;

const RENDER_TERRAIN_TILING_DESCRIPTIONS: Readonly<Record<RenderTerrainTiling, string>> = {
  [ERenderTerrainTiling.ENGINE]:
    "The terrain's details repeat as a grid, every few metres the same, as the game draws them.",
  [ERenderTerrainTiling.VARIED]:
    "The terrain's details are shifted at random over small hexagons and blended at their seams, so no repeat of the " +
    "ground lines up with the next.",
};

/**
 * @param tiling - A terrain tiling.
 * @returns What it does, in a sentence.
 */
export function explainRenderTerrainTiling(tiling: RenderTerrainTiling): string {
  return RENDER_TERRAIN_TILING_DESCRIPTIONS[tiling];
}

/** The terrain tilings, in the order they are offered. */
export const RENDER_TERRAIN_TILING_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderTerrainTiling>> = [
  { label: "Engine", value: ERenderTerrainTiling.ENGINE },
  { label: "Varied", value: ERenderTerrainTiling.VARIED },
];

/** @returns How many hexagons the tiling lays to the hex tiling demo's one, as a multiple. */
export function formatTerrainScale(scale: number): string {
  return `${formatNumber(scale, 2)}×`;
}

/** @returns How far a copy turns at most either way, a share of a half turn, in degrees. */
export function formatTerrainRotation(rotation: number): string {
  return `±${formatNumber(rotation * 180, 0)}°`;
}

/** @returns How sharply the copies meet, the exponent their nearness is raised to. */
export function formatTerrainSharpness(sharpness: number): string {
  return formatNumber(sharpness, 1);
}

/** Whether the details repeat as the game's grid, which none of the varied tiling's values tune. */
function isEngineTiling(terrain: TRenderTerrainSettings): boolean {
  return terrain.tiling === ERenderTerrainTiling.ENGINE;
}

/** How each terrain setting is set, in the settings and the Shading popover alike. */
export const RENDER_TERRAIN_SETTINGS: TRenderSettings<TRenderTerrainSettings> = {
  rotation: {
    control: ERenderSettingControl.SLIDER,
    description:
      "How far each hexagon's copy turns. The details carry painted light and shade, which turns with them and " +
      "mottles the ground, so none by default.",
    format: formatTerrainRotation,
    isInert: isEngineTiling,
    label: "Rotation",
    limits: RENDER_TERRAIN_LIMITS.rotation,
  },
  scale: {
    control: ERenderSettingControl.SLIDER,
    description:
      "How small the hexagons are: larger scatters fine repeated features such as rows of pebbles, smaller " +
      "keeps more of each detail whole.",
    format: formatTerrainScale,
    isInert: isEngineTiling,
    label: "Scale",
    limits: RENDER_TERRAIN_LIMITS.scale,
  },
  sharpness: {
    control: ERenderSettingControl.SLIDER,
    description: "How sharply the copies meet: lower blends wider, softer seams; higher keeps each copy whole.",
    format: formatTerrainSharpness,
    isInert: isEngineTiling,
    label: "Blend sharpness",
    limits: RENDER_TERRAIN_LIMITS.sharpness,
  },
  tiling: {
    control: ERenderSettingControl.CHOICE,
    explain: explainRenderTerrainTiling,
    label: "Tiling",
    options: RENDER_TERRAIN_TILING_OPTIONS,
  },
};
