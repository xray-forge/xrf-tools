import { formatPercent } from "@xrf/format";
import { ERenderImpostorMode, RenderImpostorMode } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { IRenderLimits } from "@/core/render/lib/features/render-limits";

/** The share of a level's trees whose types bake at the full quality: from half of them to every one. */
export const RENDER_IMPOSTOR_SHARE_LIMITS = { max: 1, min: 0.5, step: 0.05 } as const satisfies IRenderLimits;

const RENDER_IMPOSTOR_MODE_DESCRIPTIONS: Readonly<Record<RenderImpostorMode, string>> = {
  [ERenderImpostorMode.ENGINE]: "Each tree's impostor as the level compiler baked it: eight views around it.",
  [ERenderImpostorMode.BAKED]:
    "Baked from each tree type's own meshes as the level opens: seen from every side and from above, lit as the trees.",
};

/**
 * @param mode - An impostor mode.
 * @returns What it draws, in a sentence.
 */
export function explainRenderImpostorMode(mode: RenderImpostorMode): string {
  return RENDER_IMPOSTOR_MODE_DESCRIPTIONS[mode];
}

/** The impostor modes, in the order they are offered. */
export const RENDER_IMPOSTOR_MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderImpostorMode>> = [
  { label: "Engine", value: ERenderImpostorMode.ENGINE },
  { label: "Baked", value: ERenderImpostorMode.BAKED },
];

/** A baked atlas's texels a side, as a choice offers it. */
export type TRenderImpostorQuality = "512" | "1024" | "2048";

/** The baked atlases' sizes, in the order they are offered: Epic's far foliage's at the top. */
export const RENDER_IMPOSTOR_QUALITY_OPTIONS: ReadonlyArray<IChoiceFormRowOption<TRenderImpostorQuality>> = [
  { label: "Low", value: "512" },
  { label: "Medium", value: "1024" },
  { label: "High", value: "2048" },
];

/**
 * @param size - A baked atlas's texels a side.
 * @returns The quality offered nearest it.
 */
export function toRenderImpostorQuality(size: number): TRenderImpostorQuality {
  return size <= 768 ? "512" : size <= 1536 ? "1024" : "2048";
}

/**
 * @param quality - A baked atlas's quality.
 * @returns How it reads: its size, and how far it holds up.
 */
export function explainRenderImpostorQuality(quality: TRenderImpostorQuality): string {
  return {
    "512": "A 512 atlas for each tree type: for impostors at the game's distance or farther.",
    "1024": "A 1024 atlas for each tree type: sharp to about half the game's distance.",
    "2048":
      "A 2048 atlas for each tree type: sharp to about a quarter of the game's distance, at four times the memory.",
  }[quality];
}

/** @returns How much of a level's trees bake at the full quality. */
export function formatImpostorShare(share: number): string {
  return `${formatPercent(share)} of trees`;
}
