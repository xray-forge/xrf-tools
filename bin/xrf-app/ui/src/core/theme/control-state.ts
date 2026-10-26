import { iconButtonClasses, Theme } from "@mui/material";
import { SystemStyleObject } from "@mui/system";

/**
 * The paint a compact control takes in each of the three states it can be in.
 */
export function getControlStateSx(isActive: boolean): SystemStyleObject<Theme> {
  return {
    color: isActive ? "primary.main" : "text.primary",
    backgroundColor: "transparent",
    [`&:hover:not(.${iconButtonClasses.disabled})`]: {
      backgroundColor: "action.hover",
    },
    // These carry no border of their own, so without a ring a keyboard user has nothing marking where they are.
    "&:focus-visible": {
      outline: 1,
      outlineStyle: "solid",
      outlineColor: "primary.main",
      outlineOffset: -2,
    },
    // Terminal, and outranks active: a control that cannot be acted on must not advertise the state it would have had.
    [`&.${iconButtonClasses.disabled}`]: {
      color: "action.disabled",
      backgroundColor: "transparent",
    },
  };
}
