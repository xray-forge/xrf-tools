import { formatNumber } from "@xrf/format";
import { ERenderTerrainTiling, RenderTerrainTiling } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { IRenderLimits } from "@/core/render/lib/features/render-limits";

/**
 * The bounds each varied tiling value is offered between: narrower than the renderer takes, where the tiling reads.
 * Past four its hexagons shrink to a blur of copies; past sixteen their seams show as hard edges.
 */
export const RENDER_TERRAIN_LIMITS = {
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
