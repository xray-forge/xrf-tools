import { Theme } from "@mui/material";
import { SystemStyleObject } from "@mui/system";

import { ACCENT, RECESS_TONE, STATE, SURFACE, toSharePercent, WASH } from "./tokens";

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
