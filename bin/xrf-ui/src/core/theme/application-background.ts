import { Theme } from "@mui/material";
import { SystemStyleObject } from "@mui/system";

import { APPLICATION_BACKGROUND } from "@/core/theme/tokens";

/** Theme-colored background whose fields stay fixed while the viewport resizes or its contents scroll. */
export function getApplicationBackgroundSx(theme: Theme): SystemStyleObject<Theme> {
  return {
    backgroundColor: "background.default",
    "--xrf-background-primary-opacity": APPLICATION_BACKGROUND.primary.opacity.light,
    "--xrf-background-secondary-opacity": APPLICATION_BACKGROUND.secondary.opacity.light,
    backgroundImage: [
      `radial-gradient(ellipse ${APPLICATION_BACKGROUND.radiusX}px ${APPLICATION_BACKGROUND.radiusY}px at ${APPLICATION_BACKGROUND.secondary.x}px ${APPLICATION_BACKGROUND.secondary.y}px, color-mix(in srgb, ${(theme.vars ?? theme).palette.secondary.main} var(--xrf-background-secondary-opacity), transparent), transparent)`,
      `radial-gradient(ellipse ${APPLICATION_BACKGROUND.radiusX}px ${APPLICATION_BACKGROUND.radiusY}px at ${APPLICATION_BACKGROUND.primary.x}px ${APPLICATION_BACKGROUND.primary.y}px, color-mix(in srgb, ${(theme.vars ?? theme).palette.primary.main} var(--xrf-background-primary-opacity), transparent), transparent)`,
    ].join(", "),
    ...theme.applyStyles("dark", {
      "--xrf-background-primary-opacity": APPLICATION_BACKGROUND.primary.opacity.dark,
      "--xrf-background-secondary-opacity": APPLICATION_BACKGROUND.secondary.opacity.dark,
    }),
  };
}
