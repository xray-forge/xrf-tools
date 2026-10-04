import { ACCENT, ColorScheme, HEADER_GLOSS, RECESS_TONE, STATE, toSharePercent, WASH } from "./tokens";

/** A gradient's colour stop: sRGB channels and alpha, each from 0 to 1. */
export type TWashStop = [number, number, number, number];

/** The wash as numbers, for a painter that is not CSS: its angle in degrees and its two stops. */
export interface IWashStops {
  angle: number;
  first: TWashStop;
  last: TWashStop;
}

/**
 * @param scheme - The colour scheme shown.
 * @returns The same wash `getWashImage` paints, as its angle and stops.
 */
export function getWashStops(scheme: ColorScheme): IWashStops {
  return {
    angle: WASH.angle,
    first: toWashStop(ACCENT.secondary.main[scheme], WASH.secondary[scheme]),
    last: toWashStop(ACCENT.primary.main[scheme], WASH.primary[scheme]),
  };
}

function toWashStop(hex: string, share: number): TWashStop {
  const value: number = Number.parseInt(hex.slice(1), 16);

  return [((value >> 16) & 0xff) / 255, ((value >> 8) & 0xff) / 255, (value & 0xff) / 255, share];
}

/** Diagonal accent wash over the frame and the reading plane. */
export function getWashImage(scheme: ColorScheme): string {
  return (
    `linear-gradient(${WASH.angle}deg, ` +
    `color-mix(in srgb, ${ACCENT.secondary.main[scheme]} ${toSharePercent(WASH.secondary[scheme])}, transparent), ` +
    `color-mix(in srgb, ${ACCENT.primary.main[scheme]} ${toSharePercent(WASH.primary[scheme])}, transparent))`
  );
}

/** A well's own fill, published per scheme by `variables.ts`. It is a recess on a level, not a level of its own. */
export const WELL_FILL: string = "var(--xrf-well)";

/**
 * A recess holding input or a nested listing, as a translucent scrim so it composites over whichever level hosts it.
 */
export function getWellFill(scheme: ColorScheme): string {
  return `color-mix(in srgb, ${RECESS_TONE} ${toSharePercent(STATE.well[scheme])}, transparent)`;
}

/**
 * The gloss over the band a surface names itself in: the file header over the body, and a panel's title row.
 */
export function getHeaderGlossImage(scheme: ColorScheme): string {
  return (
    `linear-gradient(180deg, ` +
    `rgba(255, 255, 255, ${HEADER_GLOSS.highlight[scheme]}) 0%, ` +
    `rgba(255, 255, 255, 0) ${HEADER_GLOSS.fadeAt}, ` +
    `rgba(0, 0, 0, ${HEADER_GLOSS.shade[scheme]}) 100%)`
  );
}
