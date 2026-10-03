import { Box } from "@mui/material";
import { ReactElement } from "react";
import { RgbColor, RgbColorPicker } from "react-colorful";

import { RADIUS } from "@/core/theme/tokens";
import { IPickedColor } from "@/core/ui/color/color-intensity";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IColorPickerProps extends BaseComponentProps {
  value: IPickedColor;
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
    >
      <RgbColorPicker color={value} onChange={(color: RgbColor) => onChange(color)} />
    </Box>
  );
}
