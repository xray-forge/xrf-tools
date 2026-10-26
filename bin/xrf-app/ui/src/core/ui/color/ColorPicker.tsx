import { Box } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useEffect } from "react";
import { RgbColorPicker } from "react-colorful";

import { RADIUS } from "@/core/theme/tokens";
import { IPickedColor } from "@/core/ui/color/color-intensity";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { useThrottledValue } from "@/lib/react/use-throttled-value";

/**
 * @param left - A colour told or shown.
 * @param right - Another, or none.
 * @returns Whether they are one colour of the picker.
 */
function isSameColor(left: IPickedColor, right: Nullable<IPickedColor>): boolean {
  return right !== null && left.r === right.r && left.g === right.g && left.b === right.b;
}

interface IColorPickerProps extends BaseComponentProps {
  value: IPickedColor;
  /** Told the colour while it is dragged at most once an interval, and at once when it is let go. */
  onChange: (value: IPickedColor) => void;
}

/**
 * A colour's hue and shade picked by pointer: a saturation square over a hue bar, drawn in the application's style.
 */
export function ColorPicker({
  "data-testid": dataTestId = "color-picker",
  id,
  className,
  value,
  onChange,
}: IColorPickerProps): ReactElement {
  // The colour shown while it is dragged, ahead of what the owner was told.
  const { value: shown, isDrafted, change, end } = useThrottledValue(value, isSameColor, onChange);
  const endDrag = useCallback((): void => end(), [end]);

  // The picker follows a drag across the whole window, so one let go outside it ends there too.
  useEffect(() => {
    if (!isDrafted) {
      return;
    }

    window.addEventListener("pointerup", endDrag);
    window.addEventListener("pointercancel", endDrag);

    return () => {
      window.removeEventListener("pointerup", endDrag);
      window.removeEventListener("pointercancel", endDrag);
    };
  }, [isDrafted, endDrag]);

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={(theme) => ({
        "& .react-colorful": { gap: 1, height: 160, width: "100%" },
        "& .react-colorful__saturation": {
          border: `1px solid ${theme.palette.divider}`,
          borderBottom: `1px solid ${theme.palette.divider}`,
          borderRadius: `${RADIUS.md}px`,
        },
        "& .react-colorful__hue": {
          border: `1px solid ${theme.palette.divider}`,
          borderRadius: `${RADIUS.md}px`,
          flex: "0 0 10px",
        },
        "& .react-colorful__last-control": { borderRadius: `${RADIUS.md}px` },
        "& .react-colorful__pointer": {
          border: `2px solid ${theme.palette.common.white}`,
          boxShadow: "0 0 0 1px rgba(0, 0, 0, 0.6)",
          height: 14,
          width: 14,
        },
      })}
      onKeyUp={endDrag}
    >
      <RgbColorPicker color={shown} onChange={change} />
    </Box>
  );
}
