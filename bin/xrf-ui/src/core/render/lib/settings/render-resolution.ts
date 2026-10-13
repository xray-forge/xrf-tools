import { RenderViewOutput } from "@/core/ipc/types/xrf-renderer";

/**
 * What every viewport resolves its frame at before it is put into the viewport: the viewport's own size times a
 * scale, or a height in the viewport's shape.
 */
export enum ERenderResolution {
  /** The viewport's own size, times the output scale. */
  AUTO = "auto",
  HEIGHT_720 = "720",
  HEIGHT_1080 = "1080",
  HEIGHT_1440 = "1440",
  HEIGHT_2160 = "2160",
}

/** The choices offered: the viewport's own size, and the four heights worth naming. */
export const RENDER_RESOLUTIONS: ReadonlyArray<ERenderResolution> = [
  ERenderResolution.AUTO,
  ERenderResolution.HEIGHT_720,
  ERenderResolution.HEIGHT_1080,
  ERenderResolution.HEIGHT_1440,
  ERenderResolution.HEIGHT_2160,
];

/** What a viewport resolves at until somebody says otherwise. */
export const DEFAULT_RENDER_RESOLUTION: ERenderResolution = ERenderResolution.AUTO;

/** The share of the viewport's sides an automatic resolution resolves at: the least, the most and the default. */
export const RENDER_OUTPUT_SCALES = { min: 0.5, max: 2, initial: 1 } as const;

/** What a view resolves its frame at: a height asked, or a share of the viewport's sides. */
export type TRenderOutput = Pick<RenderViewOutput, "outputHeight" | "outputScale">;

/** The viewport's own size. */
export const NATIVE_RENDER_OUTPUT: TRenderOutput = { outputHeight: null, outputScale: 1 };

/**
 * Reads a stored choice, falling back to the default rather than trusting what is in storage.
 *
 * @param stored - What local storage holds, which is a string or nothing at all.
 * @returns One of the offered resolutions.
 */
export function toRenderResolution(stored: unknown): ERenderResolution {
  return RENDER_RESOLUTIONS.find((resolution: ERenderResolution) => resolution === stored) ?? DEFAULT_RENDER_RESOLUTION;
}

/**
 * Reads a stored output scale, falling back to the default for anything outside the offered range.
 *
 * @param stored - What local storage holds, which is a string or nothing at all.
 * @returns A share of the viewport's sides between the least and the most offered.
 */
export function toRenderOutputScale(stored: unknown): number {
  const scale: number = typeof stored === "string" ? Number(stored) : Number.NaN;

  return scale >= RENDER_OUTPUT_SCALES.min && scale <= RENDER_OUTPUT_SCALES.max ? scale : RENDER_OUTPUT_SCALES.initial;
}

/**
 * @param resolution - What the settings resolve every viewport at.
 * @param scale - The share of the viewport's sides an automatic resolution resolves at.
 * @returns What a native viewport resolves its frame at.
 */
export function toRenderOutput(resolution: ERenderResolution, scale: number): TRenderOutput {
  return resolution === ERenderResolution.AUTO
    ? { outputHeight: null, outputScale: scale }
    : { outputHeight: Number(resolution), outputScale: 1 };
}
