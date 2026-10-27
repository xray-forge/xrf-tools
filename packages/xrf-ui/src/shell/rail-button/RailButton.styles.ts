import { SystemStyleObject, Theme } from "#/material/styles";
import { svgIconClasses } from "#/material/svg-icon";
import { LAYOUT } from "#/theme";

/** Shared rail metrics. State and feedback are `getControlStateSx`, which every chrome toggle composes. */
export const RAIL_BUTTON_SX: SystemStyleObject<Theme> = {
  width: LAYOUT.railButtonSize,
  height: LAYOUT.railButtonSize,
  padding: 0,
  borderRadius: 1,
  [`& .${svgIconClasses.root}`]: { fontSize: LAYOUT.railButtonIconSize },
};
