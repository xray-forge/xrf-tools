import { inputBaseClasses, outlinedInputClasses } from "#/input";
import { POPOVER } from "#/theme";
import { SystemStyleObject, Theme } from "#/theme/styles";

const LINE_HEIGHT: number = POPOVER.inputHeight - POPOVER.inputPaddingY * 2;
// In `sx` a bare number pads in theme spacing units, so the padding is spelt in pixels.
const PADDING_Y: string = `${POPOVER.inputPaddingY}px`;

/**
 * A small outlined field sized for a popover row rather than a form, given to the field's own root (a `Select`, or an
 * `Autocomplete`'s input through its `input` slot). Tripled `&` outranks the theme's small height and the
 * autocomplete's own padding.
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
