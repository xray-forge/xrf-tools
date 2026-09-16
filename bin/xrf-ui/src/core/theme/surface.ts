import { Theme } from "@mui/material";
import { SystemStyleObject } from "@mui/system";

import { ACCENT, HEADER_GLOSS, RECESS_TONE, STATE, SURFACE, toSharePercent, WASH } from "./tokens";

/** One of the three opaque planes. */
export type TSurfaceLevel = keyof typeof SURFACE;

/** Palette slot carrying each level. `paper` is the overlay so every MUI `Paper` descendant is right by default. */
const LEVEL_PALETTE: Record<TSurfaceLevel, string> = {
  content: "background.default",
  frame: "background.frame",
  overlay: "background.paper",
};

/** Shared gradient formula, so the build-time first paint and the mounted application draw the same wash. */
export function getWashImage(scheme: "light" | "dark"): string {
  return (
    `linear-gradient(${WASH.angle}, ` +
    `color-mix(in srgb, ${ACCENT.secondary.main[scheme]} ${toSharePercent(WASH.secondary[scheme])}, transparent), ` +
    `color-mix(in srgb, ${ACCENT.primary.main[scheme]} ${toSharePercent(WASH.primary[scheme])}, transparent))`
  );
}

/**
 * A level and its wash.
 */
export function getSurfaceSx(level: TSurfaceLevel): (theme: Theme) => SystemStyleObject<Theme> {
  return (theme: Theme): SystemStyleObject<Theme> => {
    if (level === "overlay") {
      return { backgroundColor: LEVEL_PALETTE[level] };
    }

    return {
      backgroundColor: LEVEL_PALETTE[level],
      backgroundAttachment: "fixed",
      backgroundImage: getWashImage("light"),
      ...theme.applyStyles("dark", { backgroundImage: getWashImage("dark") }),
    };
  };
}

/**
 * The recess itself, for a well that draws its own edges.
 */
export function getWellFillSx(theme: Theme): SystemStyleObject<Theme> {
  return {
    backgroundColor: `color-mix(in srgb, ${RECESS_TONE} ${toSharePercent(STATE.well.light)}, transparent)`,
    ...theme.applyStyles("dark", {
      backgroundColor: `color-mix(in srgb, ${RECESS_TONE} ${toSharePercent(STATE.well.dark)}, transparent)`,
    }),
  };
}

/**
 * A recess on whatever level hosts it, with the border that makes it legible.
 */
export function getWellSx(theme: Theme): SystemStyleObject<Theme> {
  return {
    ...getWellFillSx(theme),
    border: "1px solid",
    borderColor: "divider",
  };
}

/** The band's gloss for one scheme, as a gradient over its flat fill. */
export function getHeaderGlossImage(scheme: "light" | "dark"): string {
  return (
    `linear-gradient(180deg, ` +
    `rgba(255, 255, 255, ${HEADER_GLOSS.highlight[scheme]}) 0%, ` +
    `rgba(255, 255, 255, 0) ${HEADER_GLOSS.fadeAt}, ` +
    `rgba(0, 0, 0, ${HEADER_GLOSS.shade[scheme]}) 100%)`
  );
}

/**
 * The band a surface names itself in: the file header over the body, and a panel's title row.
 *
 * Flat `frame` rather than the level helper - the band is chrome in its own right, not a piece of the surface behind
 * it - with a gloss of its own so it reads as a lit strip instead of a painted rectangle.
 */
export function getHeaderBandSx(theme: Theme): SystemStyleObject<Theme> {
  return {
    backgroundColor: "background.frame",
    backgroundImage: getHeaderGlossImage("light"),
    ...theme.applyStyles("dark", { backgroundImage: getHeaderGlossImage("dark") }),
  };
}
