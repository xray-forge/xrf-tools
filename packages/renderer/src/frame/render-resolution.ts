/**
 * How many pixels a viewport draws, whatever size it is shown at.
 */
export enum ERenderResolution {
  /** As many as the display has, which is what every viewport did before this was askable. */
  WINDOW = "window",
  HEIGHT_720 = "720",
  HEIGHT_1080 = "1080",
  HEIGHT_2160 = "2160",
}

/** The choices offered: the window itself, and the three heights worth naming. */
export const RENDER_RESOLUTIONS: ReadonlyArray<ERenderResolution> = [
  ERenderResolution.WINDOW,
  ERenderResolution.HEIGHT_720,
  ERenderResolution.HEIGHT_1080,
  ERenderResolution.HEIGHT_2160,
];

/** What a viewport draws until somebody says otherwise. */
export const DEFAULT_RENDER_RESOLUTION: ERenderResolution = ERenderResolution.WINDOW;

/**
 * Reads a stored choice, falling back to the default rather than trusting what is in storage.
 *
 * @param stored - What local storage holds, which is a string or nothing at all.
 * @returns One of the offered resolutions.
 */
export function toRenderResolution(stored: unknown): ERenderResolution {
  return RENDER_RESOLUTIONS.find((resolution) => resolution === stored) ?? DEFAULT_RENDER_RESOLUTION;
}

/**
 * How many device pixels a viewport draws for each css pixel it occupies.
 *
 * @param resolution - What the viewer asked for.
 * @param height - The element's height in css pixels, or zero before it has been measured.
 * @param devicePixelRatio - What the display says one css pixel is worth.
 * @returns The ratio to draw at, never zero.
 */
export function toRenderPixelRatio(resolution: ERenderResolution, height: number, devicePixelRatio: number): number {
  if (resolution === ERenderResolution.WINDOW || height <= 0) {
    return devicePixelRatio;
  }

  return Number(resolution) / height;
}
