import { inputBaseClasses, outlinedInputClasses, Theme, toggleButtonClasses } from "@mui/material";
import { SystemStyleObject } from "@mui/system";

import { WELL_FILL } from "./surface";
import { CONTROL, FORM } from "./tokens";

const CONTROL_LINE_HEIGHT: number = FORM.controlHeight - CONTROL.smallInputPaddingY * 2;
const CONTROL_RADIUS: string = `${FORM.controlRadius}px`;

/** The paint a form takes where filling it is the whole point of the screen. */
export const FORM_SURFACE_SX: SystemStyleObject<Theme> = {
  [`& .${outlinedInputClasses.root}`]: {
    borderRadius: CONTROL_RADIUS,
  },
  [`& .${outlinedInputClasses.root}.${inputBaseClasses.sizeSmall}:not(.${inputBaseClasses.multiline}) .${outlinedInputClasses.input}`]:
    {
      height: CONTROL_LINE_HEIGHT,
      minHeight: CONTROL_LINE_HEIGHT,
    },
  [`& .${toggleButtonClasses.root}.${toggleButtonClasses.root}`]: {
    height: FORM.controlHeight,
    paddingInline: `${FORM.inlineRowPaddingX}px`,
    "&:first-of-type": { borderStartStartRadius: CONTROL_RADIUS, borderEndStartRadius: CONTROL_RADIUS },
    "&:last-of-type": { borderStartEndRadius: CONTROL_RADIUS, borderEndEndRadius: CONTROL_RADIUS },
  },
  '& [data-form-row="inline"]': {
    backgroundColor: WELL_FILL,
    border: "1px solid",
    borderColor: "divider",
    borderRadius: CONTROL_RADIUS,
    padding: `${FORM.inlineRowPaddingY}px ${FORM.inlineRowPaddingX}px`,
  },
};
