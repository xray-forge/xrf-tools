import { Box } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { IEditorPanel, TEditorPanelSide } from "@/core/shell/panel/context";
import { PanelStripeButton } from "@/core/shell/panel/PanelStripeButton";
import { LAYOUT } from "@/core/theme/tokens";
import { Nullable } from "@/lib/types/general";

interface IApplicationPanelStripeProps {
  side: TEditorPanelSide;
  panels: Array<IEditorPanel>;
  activePanelId: Nullable<string>;
  /** Pinned to the bottom: what the shell owns, below what the application declared. */
  footer?: ReactNode;
  onTogglePanel: (id: string) => void;
}

/**
 * One edge of the window frame: the application's panels at the top, the shell's own controls at the
 * bottom.
 *
 * Both sides render through here so they cannot drift apart. The stripe stays put even when an
 * application declares no panels, for the same reason every route has a toolbar - a frame that changes
 * shape as you move between applications is harder to read than one that does not.
 */
export function ApplicationPanelStripe({
  side,
  panels,
  activePanelId,
  footer,
  onTogglePanel,
}: IApplicationPanelStripeProps): ReactElement {
  return (
    <Box
      sx={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 0.5,
        width: LAYOUT.railWidth,
        minWidth: LAYOUT.railWidth,
        paddingY: 1,
        ...(side === "left" ? { borderRight: 1 } : { borderLeft: 1 }),
        borderColor: "divider",
        backgroundColor: "background.paper",
      }}
    >
      {panels.map((panel: IEditorPanel) => (
        <PanelStripeButton
          key={panel.id}
          panel={panel}
          side={side}
          isActive={panel.id === activePanelId}
          onTogglePanel={onTogglePanel}
        />
      ))}

      <Box sx={{ flexGrow: 1 }} />

      {footer}
    </Box>
  );
}
