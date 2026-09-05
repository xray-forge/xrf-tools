import { ButtonBase } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { LAYOUT } from "@/core/theme/tokens";

/** Windows tints only the close button, and keeps the same red in both schemes. */
const CLOSE_HOVER: string = "#c42b1c";
const CLOSE_ACTIVE: string = "#b7291b";

interface IWindowControlButtonProps {
  label: string;
  /** Marks the button that ends the session, which the system paints differently for that reason. */
  isDanger?: boolean;
  children: ReactNode;
  onClick: () => void;
}

export function WindowControlButton({ label, isDanger, children, onClick }: IWindowControlButtonProps): ReactElement {
  return (
    <ButtonBase
      aria-label={label}
      disableRipple={true}
      sx={{
        width: LAYOUT.windowControlWidth,
        height: LAYOUT.titleBarHeight,
        flexShrink: 0,
        color: "text.primary",
        // A caption button lights up the instant it is entered. Easing it makes the window feel slow
        // before anything inside it has even been asked to do work.
        transition: "none",
        "&:hover": {
          backgroundColor: isDanger ? CLOSE_HOVER : "action.hover",
          color: isDanger ? "#ffffff" : "text.primary",
        },
        "&:active": {
          backgroundColor: isDanger ? CLOSE_ACTIVE : "action.selected",
        },
        // These carry no border of their own, so hover colour is the only thing marking them - which
        // leaves a keyboard user with nothing.
        "&:focus-visible": {
          outline: 1,
          outlineStyle: "solid",
          outlineColor: "primary.main",
          outlineOffset: -2,
        },
      }}
      onClick={onClick}
    >
      {children}
    </ButtonBase>
  );
}
