import { BaseComponentProps } from "@xrf/ui/element-types";
import { IconButton } from "@xrf/ui/material/icon-button";
import { Tooltip } from "@xrf/ui/material/tooltip";
import { RAIL_BUTTON_SX, RailButton } from "@xrf/ui/shell/rail-button";
import { getControlStateSx } from "@xrf/ui/theme/sx";
import { ReactElement, useCallback } from "react";

import { IShellPanel, TShellPanelSide } from "@/core/shell/panels/shell-panel";

interface IPanelStripeButtonProps extends BaseComponentProps {
  panel: IShellPanel;
  side: TShellPanelSide;
  isActive: boolean;
  /** Drawn as the shell's own controls are, in the rail's gradient, so what the shell owns reads apart. */
  isShellOwned?: boolean;
  onTogglePanel: (id: string) => void;
}

/**
 * One control in a panel stripe, opening its panel or closing it.
 */
export function PanelStripeButton({
  "data-testid": dataTestId = "panel-stripe-button",
  id,
  className,
  panel,
  side,
  isActive,
  isShellOwned = false,
  onTogglePanel,
}: IPanelStripeButtonProps): ReactElement {
  const placement: "left" | "right" = side === "left" ? "right" : "left";

  const onClick = useCallback(() => onTogglePanel(panel.id), [onTogglePanel, panel.id]);

  if (isShellOwned) {
    return (
      <RailButton
        label={panel.label}
        icon={panel.icon}
        tooltipPlacement={placement}
        isSelected={isActive}
        onClick={onClick}
      />
    );
  }

  return (
    <Tooltip title={panel.label} placement={placement}>
      <IconButton
        data-testid={dataTestId}
        aria-label={panel.label}
        aria-pressed={isActive}
        id={id}
        className={className}
        sx={[RAIL_BUTTON_SX, getControlStateSx(isActive)]}
        onClick={onClick}
      >
        {panel.icon}
      </IconButton>
    </Tooltip>
  );
}
