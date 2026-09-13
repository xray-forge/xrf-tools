import { Theme } from "@mui/material";
import { SystemStyleObject } from "@mui/system";

import { getControlBackgroundSx } from "@/core/theme/control-background";
import { LAYOUT } from "@/core/theme/tokens";

/** Shared rail metrics and brand background on hover. */
export function getRailButtonSx(theme: Theme): SystemStyleObject<Theme> {
  return {
    ...getControlBackgroundSx(theme),
    width: LAYOUT.railButtonSize,
    height: LAYOUT.railButtonSize,
    padding: 0,
    borderRadius: 1,
    "& .MuiSvgIcon-root": { fontSize: LAYOUT.railButtonIconSize },
    "&:not(:hover), &.Mui-disabled": { backgroundImage: "none" },
  };
}
