import { BaseComponentProps } from "@xrf/ui/element-types";
import { IconButton } from "@xrf/ui/material/icon-button";
import { Tooltip } from "@xrf/ui/material/tooltip";
import { RAIL_BUTTON_SX } from "@xrf/ui/shell/rail-button";
import { getControlStateSx } from "@xrf/ui/theme/sx";
import { ReactElement } from "react";

import { IEditorPanel, TEditorPanelSide } from "@/core/shell/editor-shell";

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
        sx={[RAIL_BUTTON_SX, getControlStateSx(isActive)]}
        onClick={() => onTogglePanel(panel.id)}
      >
        {panel.icon}
      </IconButton>
    </Tooltip>
  );
}
