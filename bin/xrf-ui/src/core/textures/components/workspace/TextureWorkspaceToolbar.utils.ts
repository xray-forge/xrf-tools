/** What every control that only makes sense over a lit body says while the flat picture is on screen. */
export const SURFACE_ONLY: string = "Lit surface only";

/**
 * Why shading with the bump pair is unavailable, in the order a person would ask.
 *
 * @param isSurface - Whether the lit body is on screen at all.
 * @param hasBump - Whether the open texture declares a pair.
 * @param isLit - Whether a light is shading the body.
 * @returns The reason, or undefined when the toggle is available.
 */
export function describeUnavailableBump(isSurface: boolean, hasBump: boolean, isLit: boolean): string | undefined {
  if (!isSurface) {
    return SURFACE_ONLY;
  }

  if (!hasBump) {
    return "This texture declares no bump pair";
  }

  return isLit ? undefined : "Nothing to shade with the light off";
}
