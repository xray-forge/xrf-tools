import { Theme } from "@mui/material";
import { SystemStyleObject } from "@mui/system";

import { APPLICATION_BACKGROUND } from "@/core/theme/tokens";

import { getApplicationBackgroundImage } from "./application-background-image";

/** Theme-colored background whose fields stay fixed while the viewport resizes or its contents scroll. */
export function getApplicationBackgroundSx(theme: Theme): SystemStyleObject<Theme> {
  return {
    backgroundColor: "background.default",
    "--xrf-background-primary-opacity": APPLICATION_BACKGROUND.primary.opacity.light,
    "--xrf-background-secondary-opacity": APPLICATION_BACKGROUND.secondary.opacity.light,
    backgroundImage: getApplicationBackgroundImage({
      primary: (theme.vars ?? theme).palette.primary.main,
      secondary: (theme.vars ?? theme).palette.secondary.main,
      primaryOpacity: "var(--xrf-background-primary-opacity)",
      secondaryOpacity: "var(--xrf-background-secondary-opacity)",
    }),
    ...theme.applyStyles("dark", {
      "--xrf-background-primary-opacity": APPLICATION_BACKGROUND.primary.opacity.dark,
      "--xrf-background-secondary-opacity": APPLICATION_BACKGROUND.secondary.opacity.dark,
    }),
  };
}
