import { Theme } from "@mui/material";
import { SystemStyleObject } from "@mui/system";

import { CONTROL_BACKGROUND } from "@/core/theme/tokens";

/** One end of the sweep: the surface the control sits on, carrying a share of one accent. */
function toControlSurface(accent: string, share: string): string {
  return `color-mix(in srgb, ${accent} ${share}, var(--xrf-control-base))`;
}

/**
 * The window's two accents swept across a control's own box, for a control that would otherwise read as a hole in it.
 */
export function getControlBackgroundSx(theme: Theme): SystemStyleObject<Theme> {
  const palette = (theme.vars ?? theme).palette;

  return {
    "--xrf-control-base": palette.background.default,
    "--xrf-control-primary-share": CONTROL_BACKGROUND.primary.share.light,
    "--xrf-control-secondary-share": CONTROL_BACKGROUND.secondary.share.light,
    backgroundImage:
      `linear-gradient(${CONTROL_BACKGROUND.angle}, ` +
      `${toControlSurface(palette.secondary.main, "var(--xrf-control-secondary-share)")}, ` +
      `${toControlSurface(palette.primary.main, "var(--xrf-control-primary-share)")})`,
    "&:hover": {
      "--xrf-control-primary-share": CONTROL_BACKGROUND.primary.hoverShare.light,
      "--xrf-control-secondary-share": CONTROL_BACKGROUND.secondary.hoverShare.light,
    },
    ...theme.applyStyles("dark", {
      "--xrf-control-primary-share": CONTROL_BACKGROUND.primary.share.dark,
      "--xrf-control-secondary-share": CONTROL_BACKGROUND.secondary.share.dark,
      "&:hover": {
        "--xrf-control-primary-share": CONTROL_BACKGROUND.primary.hoverShare.dark,
        "--xrf-control-secondary-share": CONTROL_BACKGROUND.secondary.hoverShare.dark,
      },
    }),
  };
}
