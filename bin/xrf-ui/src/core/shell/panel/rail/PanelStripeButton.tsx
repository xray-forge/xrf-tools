import { IconButton, Tooltip } from "@mui/material";
import { ReactElement } from "react";

import { IEditorPanel, TEditorPanelSide } from "@/core/shell/editor-shell";
import { getRailButtonSx } from "@/core/shell/panel/rail/RailButton.styles";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IPanelStripeButtonProps extends BaseComponentProps {
  panel: IEditorPanel;
  side: TEditorPanelSide;
  isActive: boolean;
  onTogglePanel: (id: string) => void;
}

/**
 * One control in a panel stripe.
 */
export function PanelStripeButton({
  "data-testid": dataTestId = "panel-stripe-button",
  id,
  className,
  panel,
  side,
  isActive,
  onTogglePanel,
}: IPanelStripeButtonProps): ReactElement {
  return (
    <Tooltip title={panel.label} placement={side === "left" ? "right" : "left"}>
      <IconButton
        data-testid={dataTestId}
        aria-label={panel.label}
        aria-pressed={isActive}
        id={id}
        className={className}
        sx={[
          getRailButtonSx,
          {
            color: isActive ? "primary.main" : "text.secondary",
            backgroundColor: isActive ? "action.selected" : "transparent",
          },
        ]}
        onClick={() => onTogglePanel(panel.id)}
      >
        {panel.icon}
      </IconButton>
    </Tooltip>
  );
}
