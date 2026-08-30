import { Box } from "@mui/material";
import { ReactElement } from "react";

import { IEditorPanel, TEditorPanelSide } from "@/core/shell/panel/context";
import { PanelResizer } from "@/core/shell/panel/PanelResizer";
import { Nullable } from "@/lib/types/general";

export interface IApplicationPanelSlotProps {
  side: TEditorPanelSide;
  panel: Nullable<IEditorPanel>;
  width: number;
  onResize: (width: number) => void;
}

/**
 * The docked panel on one side of the content, and the handle that sizes it.
 *
 * Two boxes rather than one: the scrolling half has to be a separate child, or the resizer - absolutely
 * positioned to cost no width - would scroll away with the panel's content.
 */
export function ApplicationPanelSlot({
  side,
  panel,
  width,
  onResize,
}: IApplicationPanelSlotProps): ReactElement | null {
  if (!panel) {
    return null;
  }

  return (
    <Box
      data-testid={`application-panel-slot-${side}`}
      sx={{
        position: "relative",
        display: "flex",
        flexDirection: "column",
        width,
        minWidth: width,
        minHeight: 0,
        ...(side === "left" ? { borderRight: 1 } : { borderLeft: 1 }),
        borderColor: "divider",
        backgroundColor: "background.default",
      }}
    >
      <Box sx={{ flexGrow: 1, minHeight: 0, overflowY: "auto" }}>{panel.render()}</Box>

      <PanelResizer side={side} width={width} onResize={onResize} />
    </Box>
  );
}
