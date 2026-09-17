import { ACCENT, ColorScheme, HEADER_GLOSS, RECESS_TONE, STATE, toSharePercent, WASH } from "./tokens";

/** Diagonal accent wash over the frame and the reading plane. */
export function getWashImage(scheme: ColorScheme): string {
  return (
    `linear-gradient(${WASH.angle}, ` +
    `color-mix(in srgb, ${ACCENT.secondary.main[scheme]} ${toSharePercent(WASH.secondary[scheme])}, transparent), ` +
    `color-mix(in srgb, ${ACCENT.primary.main[scheme]} ${toSharePercent(WASH.primary[scheme])}, transparent))`
  );
}

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
