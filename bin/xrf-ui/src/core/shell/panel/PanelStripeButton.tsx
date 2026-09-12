import { IconButton, Tooltip } from "@mui/material";
import { ReactElement } from "react";

import { IEditorPanel, TEditorPanelSide } from "@/core/shell/editor-shell";
import { LAYOUT } from "@/core/theme/tokens";

interface IPanelStripeButtonProps {
  panel: IEditorPanel;
  side: TEditorPanelSide;
  isActive: boolean;
  onTogglePanel: (id: string) => void;
}

/**
 * One control in a panel stripe.
 */
export function PanelStripeButton({ panel, side, isActive, onTogglePanel }: IPanelStripeButtonProps): ReactElement {
  return (
    <Tooltip title={panel.label} placement={side === "left" ? "right" : "left"}>
      <IconButton
        aria-label={panel.label}
        aria-pressed={isActive}
        sx={{
          width: LAYOUT.railButtonSize,
          height: LAYOUT.railButtonSize,
          padding: 0,
          borderRadius: 1,
          "& .MuiSvgIcon-root": { fontSize: LAYOUT.railButtonIconSize },
          color: isActive ? "primary.main" : "text.secondary",
          backgroundColor: isActive ? "action.selected" : "transparent",
        }}
        onClick={() => onTogglePanel(panel.id)}
      >
        {panel.icon}
      </IconButton>
    </Tooltip>
  );
}
