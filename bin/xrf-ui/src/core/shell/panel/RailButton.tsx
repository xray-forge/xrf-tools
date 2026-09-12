import { IconButton, Tooltip } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { LAYOUT } from "@/core/theme/tokens";

interface IRailButtonProps {
  isSelected?: boolean;
  isDisabled?: boolean;
  label: string;
  icon: ReactNode;
  onClick: () => void;
}

/**
 * A control on the rail that acts rather than opening a panel.
 *
 * Same metrics as `PanelStripeButton` on purpose - Home and the theme toggle sit in the same column as
 * the panel controls, so anything that made them a different size would read as a different kind of
 * thing. Wrapped in a span because a disabled button cannot receive the tooltip's events.
 */
export function RailButton({ isSelected, isDisabled, label, icon, onClick }: IRailButtonProps): ReactElement {
  return (
    <Tooltip describeChild title={label} placement={"right"}>
      <span>
        <IconButton
          aria-label={label}
          disabled={isDisabled}
          sx={{
            width: LAYOUT.railButtonSize,
            height: LAYOUT.railButtonSize,
            padding: 0,
            borderRadius: 1,
            "& .MuiSvgIcon-root": { fontSize: LAYOUT.railButtonIconSize },
            color: isSelected ? "primary.main" : "text.secondary",
            backgroundColor: isSelected ? "action.selected" : "transparent",
          }}
          onClick={onClick}
        >
          {icon}
        </IconButton>
      </span>
    </Tooltip>
  );
}
