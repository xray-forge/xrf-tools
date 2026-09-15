import { iconButtonClasses, svgIconClasses, Theme } from "@mui/material";
import { SystemStyleObject } from "@mui/system";

import { LAYOUT } from "@/core/theme/tokens";

/** Shared rail metrics. Feedback is the shared hover scrim, not a background the control paints for itself. */
export function getRailButtonSx(theme: Theme): SystemStyleObject<Theme> {
  return {
    width: LAYOUT.railButtonSize,
    height: LAYOUT.railButtonSize,
    padding: 0,
    borderRadius: 1,
    [`& .${svgIconClasses.root}`]: { fontSize: LAYOUT.railButtonIconSize },
    [`&:hover:not(.${iconButtonClasses.disabled})`]: {
      backgroundColor: (theme.vars ?? theme).palette.action.hover,
    },
  };
}
