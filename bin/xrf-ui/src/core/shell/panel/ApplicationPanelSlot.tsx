import { Box } from "@mui/material";
import { ReactElement } from "react";

import { IEditorPanel, TEditorPanelSide } from "@/core/shell/editor-shell";
import { PanelResizer } from "@/core/shell/panel/PanelResizer";
import { getSurfaceSx } from "@/core/theme/surface";
import { cn } from "@/lib/dom/dom-name";
import { Nullable } from "@/lib/types/general";

interface IApplicationPanelSlotProps {
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
      data-panel-side={side}
      className={cn("relative flex min-h-0 flex-col border-divider", side === "left" ? "border-r" : "border-l")}
      // Measured rather than authored: the slot's width is whatever the drag and the window budget left it.
      style={{ minWidth: width, width }}
      sx={getSurfaceSx("frame")}
    >
      <div className={"min-h-0 grow overflow-y-auto"}>{panel.render()}</div>

      <PanelResizer side={side} width={width} onResize={onResize} />
    </Box>
  );
}
