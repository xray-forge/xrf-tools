import { default as HelpOutlineIcon } from "@mui/icons-material/HelpOutlineOutlined";
import { ButtonBase, Tooltip } from "@mui/material";
import { ReactElement } from "react";

import { mergeSx } from "@/core/theme/merge-sx";
import { LAYOUT } from "@/core/theme/tokens";
import { StyledComponentProps } from "@/lib/dom/element-types";

export interface IApplicationHelpButtonProps extends StyledComponentProps {
  onClick: () => void;
}

/**
 * The caption-row help affordance, styled like the window controls it sits beside.
 *
 * It carries no `data-tauri-drag-region` for the same reason they do not: a clickable element without
 * a region of its own stops the drag walk, and adding one would make the button drag the window.
 */
export function ApplicationHelpButton({
  "data-testid": dataTestId = "application-help-button",
  id,
  className,
  sx,
  onClick,
}: IApplicationHelpButtonProps): ReactElement {
  return (
    <Tooltip title={"Help (F1)"}>
      <ButtonBase
        data-testid={dataTestId}
        id={id}
        className={className}
        aria-label={"Help"}
        disableRipple={true}
        sx={mergeSx(
          {
            width: LAYOUT.windowControlWidth,
            height: LAYOUT.titleBarHeight,
            flexShrink: 0,
            color: "text.secondary",
            transition: "none",
            "&:hover": {
              backgroundColor: "action.hover",
              color: "text.primary",
            },
            "&:active": {
              backgroundColor: "action.selected",
            },
            "&:focus-visible": {
              outline: 1,
              outlineStyle: "solid",
              outlineColor: "primary.main",
              outlineOffset: -2,
            },
          },
          sx
        )}
        onClick={onClick}
      >
        <HelpOutlineIcon sx={{ fontSize: 16 }} />
      </ButtonBase>
    </Tooltip>
  );
}
