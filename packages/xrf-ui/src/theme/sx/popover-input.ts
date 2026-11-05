import { inputBaseClasses, outlinedInputClasses } from "#/material/input";
import { SystemStyleObject, Theme } from "#/material/styles";
import { POPOVER } from "#/theme";

const LINE_HEIGHT: number = POPOVER.inputHeight - POPOVER.inputPaddingY * 2;
// In `sx` a bare number pads in theme spacing units, so the padding is spelt in pixels.
const PADDING_Y: string = `${POPOVER.inputPaddingY}px`;

/**
 * A small outlined field as a popover row holds it, given to the field's own root (a `Select`, or an `Autocomplete`'s
 * input through its `input` slot): the row's type and the height of its choices, where the theme sizes every small field
 * as a form's. Tripled `&` outranks the theme's small height and the autocomplete's own padding.
 */
export const POPOVER_INPUT_SX: SystemStyleObject<Theme> = {
  "&&&": {
    fontSize: "0.75rem",
  },
  [`&&&.${inputBaseClasses.sizeSmall}`]: {
    paddingTop: 0,
    paddingBottom: 0,
  },
  [`&&&.${inputBaseClasses.sizeSmall} .${outlinedInputClasses.input}`]: {
    height: LINE_HEIGHT,
    minHeight: LINE_HEIGHT,
    paddingTop: PADDING_Y,
    paddingBottom: PADDING_Y,
  },
};
